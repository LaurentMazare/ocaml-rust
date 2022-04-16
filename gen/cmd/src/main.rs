mod syntax;
use crate::syntax::api::{ApiItem, Lang, ModItem};
use crate::syntax::file::File;
use clap::Parser;
use std::io::{Read, Write};

const HEADER: &str = r#"
type isize = int;;
type i64 = Int64.t;;
type f64 = float;;

"#;

fn read_to_string<P>(path: &P) -> Result<String, std::io::Error>
where
    P: AsRef<std::path::Path>,
{
    let path = path.as_ref();
    let bytes = if path == std::path::Path::new("-") {
        let mut data = vec![];
        let _ = std::io::stdin().read_to_end(&mut data)?;
        data
    } else {
        std::fs::read(path)?
    };
    match String::from_utf8(bytes) {
        Ok(string) => Ok(string),
        Err(err) => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string())),
    }
}

/// Generate the OCaml side of the bindings
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Rust file to read
    #[clap(short, long)]
    rust_file: String,

    /// OCaml file to generate
    #[clap(short, long)]
    ocaml_file: String,
}

fn capitalize(s: &str) -> String {
    let mut s = s.chars();
    match s.next() {
        None => "".to_string(),
        Some(c) => format!("{}{}", c.to_uppercase(), s.collect::<String>()),
    }
}

fn try_main(args: Args) -> Result<(), syntax::Error> {
    let rust_source = read_to_string(&args.rust_file)?;
    proc_macro2::fallback::force();
    let file: File = syn::parse_str(&rust_source)?;
    let mut w = std::fs::File::create(args.ocaml_file)?;
    w.write(HEADER.as_bytes())?;
    for api in file.apis.iter() {
        writeln!(w, "module {} = struct", capitalize(&api.ident.to_string()))?;
        for api_item in api.api_items.iter() {
            match api_item {
                ApiItem::ForeignMod { .. } => {}
                ApiItem::Enum(e) => {
                    writeln!(w, "  type {} =", syntax::api::ocamlize(&e.ident.to_string()))?;
                    for variant in e.variants.iter() {
                        let variant_ident = capitalize(&variant.ident.to_string());
                        let args = match &variant.fields {
                            syn::Fields::Unit => "".to_string(),
                            syn::Fields::Unnamed(u) => {
                                let args: Result<Vec<String>, syntax::Error> = u
                                    .unnamed
                                    .iter()
                                    .map(|x| {
                                        Ok(syntax::api::Type::parse_type(&x.ty)?.to_ocaml_string())
                                    })
                                    .collect();
                                let args = args?.join(" * ");
                                format!(" of {args}")
                            }
                            syn::Fields::Named(n) => {
                                let args: Result<Vec<String>, syntax::Error> = n
                                    .named
                                    .iter()
                                    .map(|x| {
                                        let field_ident = match &x.ident {
                                            None => Err(syn::Error::new_spanned(
                                                &x,
                                                format!(
                                                    "struct with unnamed field {} in enum",
                                                    variant_ident
                                                ),
                                            ))?,
                                            Some(ident) => ident.to_string(),
                                        };
                                        let ty =
                                            syntax::api::Type::parse_type(&x.ty)?.to_ocaml_string();
                                        Ok(format!("{field_ident}: {ty}"))
                                    })
                                    .collect();
                                let args = args?.join("; ");
                                format!(" of {{ {args} }}")
                            }
                        };
                        writeln!(w, "  | {variant_ident}{args}")?;
                    }
                    writeln!(w, "  [@@boxed];;")?;
                }
                ApiItem::Struct(s) => {
                    writeln!(w, "  type {} = {{", syntax::api::ocamlize(&s.ident.to_string()))?;
                    for field in s.fields.iter() {
                        let ident = match &field.ident {
                            None => Err(syn::Error::new_spanned(
                                &field,
                                format!("struct with unnamed field {}", s.ident),
                            ))?,
                            Some(ident) => ident,
                        };
                        let ty = syntax::api::Type::parse_type(&field.ty)?.to_ocaml_string();
                        writeln!(w, "    {ident}: {ty};")?;
                    }
                    writeln!(w, "  }} [@@boxed];;")?;
                }
                ApiItem::Type(i) => {
                    writeln!(w, "  type {};;", syntax::api::ocamlize(&i.ident.to_string()))?;
                }
                ApiItem::Other(_) => {}
            }
        }
        for api_item in api.api_items.iter() {
            match api_item {
                ApiItem::ForeignMod { lang: Lang::Rust, items, .. } => {
                    for item in items {
                        match item {
                            ModItem::Fn { ident, args, output } => {
                                let args = if args.len() > 0 {
                                    let args: Result<Vec<std::string::String>, syn::parse::Error> =
                                        args.iter()
                                            .map(|(_ident, _ty, typ)| Ok(typ.to_ocaml_string()))
                                            .collect();
                                    args?.join(" -> ")
                                } else {
                                    "unit".to_string()
                                };
                                let output = output.to_ocaml_string();
                                writeln!(w, "  external {}", ident)?;
                                writeln!(w, "    : {} -> {}", args, output)?;
                                writeln!(w, "    = \"{}\"\n  ;;\n", api.c_fn_name(&ident))?;
                            }
                        }
                    }
                }
                ApiItem::ForeignMod { lang: Lang::OCaml, .. } => {}
                ApiItem::Enum(_) => {}
                ApiItem::Struct(_) => {}
                ApiItem::Type(_) => {}
                ApiItem::Other(_) => {}
            }
        }
        writeln!(w, "end")?;
    }
    Ok(())
}

fn main() {
    let args = Args::parse();
    if let Err(err) = try_main(args) {
        let _ = writeln!(std::io::stderr(), "rust-ocaml: {:?}", err);
        std::process::exit(1)
    }
}
