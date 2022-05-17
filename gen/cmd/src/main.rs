mod syntax;
use crate::syntax::api::{ApiItem, Lang, ModItem};
use crate::syntax::file::File;
use clap::Parser;
use std::collections::BTreeMap;
use std::io::{Read, Write};
use syn::Attribute; // TODO : Add compact to what was the Header before

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

fn ocaml_deriving(attrs: &[Attribute]) -> String {
    let deriving = attrs
        .iter()
        .flat_map(|attr| {
            if syntax::api::attr_is_ocaml_deriving(attr) {
                match &attr.tokens.clone().into_iter().collect::<Vec<_>>()[..] {
                    [proc_macro2::TokenTree::Group(group)] => group
                        .stream()
                        .into_iter()
                        .filter_map(|elem| match elem {
                            proc_macro2::TokenTree::Ident(ident) => Some(ident.to_string()),
                            _ => None,
                        })
                        .collect(),
                    _ => vec![],
                }
            } else {
                vec![]
            }
        })
        .collect::<Vec<_>>();
    if deriving.is_empty() {
        "".to_string()
    } else {
        format!("[@@deriving {}]", deriving.join(","))
    }
}

struct InScope {
    items: Vec<ModItem>,
    inner: BTreeMap<String, InScope>,
}

impl InScope {
    fn new() -> Self {
        Self { items: vec![], inner: BTreeMap::new() }
    }

    fn insert(&mut self, item: ModItem) {
        fn insert_loop(s: &mut InScope, item: ModItem, index: usize) {
            let namespace = match &item {
                ModItem::Fn { attrs, .. } => attrs.namespace.as_ref().unwrap(),
            };
            if index >= namespace.len() {
                s.items.push(item)
            } else {
                let key = namespace[index].to_string();
                let s = s.inner.entry(key).or_insert_with(InScope::new);
                insert_loop(s, item, index + 1)
            }
        }
        let attrs_is_none = match &item {
            ModItem::Fn { attrs, .. } => attrs.namespace.is_none(),
        };
        if attrs_is_none {
            self.items.push(item)
        } else {
            insert_loop(self, item, 0)
        }
    }

    fn write<W: Write>(
        &self,
        w: &mut W,
        api_ident: &proc_macro2::Ident,
    ) -> Result<(), syntax::Error> {
        for item in self.items.iter() {
            match item {
                ModItem::Fn { ident, args, output, attrs } => {
                    let args = if !args.is_empty() {
                        let args: Result<Vec<std::string::String>, syn::parse::Error> = args
                            .iter()
                            .map(|(_ident, _ty, typ)| Ok(typ.to_ocaml_string()))
                            .collect();
                        args?.join(" -> ")
                    } else {
                        "unit".to_string()
                    };
                    let output = output.1.to_ocaml_string();
                    writeln!(w, "  external {}", ident)?;
                    writeln!(w, "    : {} -> {}", args, output)?;
                    writeln!(
                        w,
                        "    = \"{}\"\n  ;;\n",
                        crate::syntax::expand::c_fn_name(
                            api_ident,
                            ident,
                            attrs.namespace.as_ref()
                        )
                    )?;
                }
            }
        }
        for (k, v) in self.inner.iter() {
            writeln!(w, "module {} = struct", capitalize(k))?;
            v.write(w, api_ident)?;
            writeln!(w, "end")?;
        }
        Ok(())
    }
}

fn try_main(args: Args) -> Result<(), syntax::Error> {
    let rust_source = read_to_string(&args.rust_file)?;
    proc_macro2::fallback::force();
    let file: File = syn::parse_str(&rust_source)?;
    let mut w = std::fs::File::create(args.ocaml_file)?;
    for api in file.apis.into_iter() {
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
                                format!(" of {}", args)
                            }
                            syn::Fields::Named(n) => {
                                let args: Result<Vec<String>, syntax::Error> = n
                                    .named
                                    .iter()
                                    .map(|x| {
                                        let field_ident = match &x.ident {
                                            None => {
                                                let msg = format!(
                                                    "struct with unnamed field {} in enum",
                                                    variant_ident
                                                );
                                                return Err(syn::Error::new_spanned(&x, msg).into());
                                            }
                                            Some(ident) => ident.to_string(),
                                        };
                                        let ty =
                                            syntax::api::Type::parse_type(&x.ty)?.to_ocaml_string();
                                        Ok(format!("{}: {}", field_ident, ty))
                                    })
                                    .collect();
                                let args = args?.join("; ");
                                format!(" of {{ {} }}", args)
                            }
                        };
                        writeln!(w, "  | {}{}", variant_ident, args)?;
                    }
                    let deriving = ocaml_deriving(&e.attrs);
                    writeln!(w, "  [@@boxed]{};;", deriving)?;
                }
                ApiItem::Struct(s) => {
                    writeln!(w, "  type {} = {{", syntax::api::ocamlize(&s.ident.to_string()))?;
                    for field in s.fields.iter() {
                        let ident = match &field.ident {
                            None => {
                                return Err(syn::Error::new_spanned(
                                    &field,
                                    format!("struct with unnamed field {}", s.ident),
                                )
                                .into())
                            }
                            Some(ident) => ident,
                        };
                        let ty = syntax::api::Type::parse_type(&field.ty)?.to_ocaml_string();
                        writeln!(w, "    {}: {};", ident, ty)?;
                    }
                    let deriving = ocaml_deriving(&s.attrs);
                    writeln!(w, "  }} [@@boxed]{};;", deriving)?;
                }
                ApiItem::Type(i) => {
                    writeln!(w, "  type {};;", syntax::api::ocamlize(&i.ident.to_string()))?;
                }
                ApiItem::Include(include) => {
                    writeln!(w, "{}", include)?;
                }
                ApiItem::Other(_) => {}
            }
        }
        let mut in_scope = InScope::new();
        for api_item in api.api_items.into_iter() {
            match api_item {
                ApiItem::ForeignMod { lang: Lang::Rust, items, .. } => {
                    for item in items {
                        in_scope.insert(item)
                    }
                }
                ApiItem::ForeignMod { lang: Lang::OCaml, .. }
                | ApiItem::Include(_)
                | ApiItem::Enum(_)
                | ApiItem::Struct(_)
                | ApiItem::Type(_)
                | ApiItem::Other(_) => {}
            }
        }
        in_scope.write(&mut w, &api.ident)?;
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
