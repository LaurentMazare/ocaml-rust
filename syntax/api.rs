use quote::{quote, ToTokens};
use syn::parse::{Error, Parse, ParseStream, Result};
use syn::spanned::Spanned;
use syn::{braced, token, Abi, Attribute, Ident, Token};

#[allow(dead_code)]
pub fn ocamlize(s: &str) -> String {
    let mut res = vec![];
    for (i, c) in s.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 {
                res.push('_');
            }
            res.push(c.to_ascii_lowercase());
        } else {
            res.push(c)
        }
    }
    res.into_iter().collect()
}

#[derive(Debug)]
pub enum Type {
    Unit,
    Ident(proc_macro2::Ident),
    Tuple(Vec<Type>),
    VecArray(Box<Type>),
    VecList(Box<Type>),
    RustResult(Box<Type>),
    BigArray1(Box<Type>),
    Option(Box<Type>),
    Result(Box<Type>, Box<Type>),
    Fn0(Box<Type>),
    Fn1(Box<Type>, Box<Type>),
}

impl Type {
    // It would be nice for this to be extensible and not requiring introducing
    // new matched cases for supporting new types.
    pub fn parse_type(ty: &syn::Type) -> Result<Type> {
        match ty {
            syn::Type::Path(ty) => {
                let path = &ty.path;
                if ty.qself.is_none() && path.leading_colon.is_none() && !path.segments.is_empty() {
                    let segment = &path.segments.last().unwrap();
                    let ident = segment.ident.clone();
                    match &segment.arguments {
                        syn::PathArguments::None => return Ok(Type::Ident(ident)),
                        syn::PathArguments::AngleBracketed(
                            syn::AngleBracketedGenericArguments { args, .. },
                        ) if args.len() == 1 => {
                            if let syn::GenericArgument::Type(ty) = &args[0] {
                                let ty = Self::parse_type(ty)?;
                                if ident == "Option" {
                                    return Ok(Type::Option(Box::new(ty)));
                                }
                                if ident == "Vec" || ident == "VecArray" {
                                    return Ok(Type::VecArray(Box::new(ty)));
                                }
                                if ident == "VecList" {
                                    return Ok(Type::VecList(Box::new(ty)));
                                }
                                if ident == "RustResult" {
                                    return Ok(Type::RustResult(Box::new(ty)));
                                }
                                if ident == "BigArray1" {
                                    return Ok(Type::BigArray1(Box::new(ty)));
                                }
                                if ident == "Fn0" {
                                    return Ok(Type::Fn0(Box::new(ty)));
                                }
                                if ident == "Box" {
                                    return Ok(ty);
                                }
                            }
                        }
                        syn::PathArguments::AngleBracketed(
                            syn::AngleBracketedGenericArguments { args, .. },
                        ) if args.len() == 2 => {
                            if let syn::GenericArgument::Type(ty0) = &args[0] {
                                if let syn::GenericArgument::Type(ty1) = &args[1] {
                                    let ty0 = Box::new(Self::parse_type(ty0)?);
                                    let ty1 = Box::new(Self::parse_type(ty1)?);
                                    if ident == "Result" {
                                        return Ok(Type::Result(ty0, ty1));
                                    }
                                    if ident == "Fn1" {
                                        return Ok(Type::Fn1(ty0, ty1));
                                    }
                                }
                            }
                        }

                        syn::PathArguments::AngleBracketed(_)
                        | syn::PathArguments::Parenthesized(_) => {}
                    }
                }
            }
            syn::Type::Tuple(tuple) => {
                let v: Result<Vec<Self>> = tuple.elems.iter().map(Self::parse_type).collect();
                return Ok(Self::Tuple(v?));
            }
            syn::Type::Reference(type_reference) => return Self::parse_type(&type_reference.elem),
            syn::Type::Slice(slice) => {
                let ty = Type::parse_type(&slice.elem)?;
                return Ok(Type::VecArray(Box::new(ty)));
            }
            _ => {}
        }
        Err(Error::new_spanned(ty, format!("unsupported type {}", ty.to_token_stream())))
    }

    #[allow(dead_code)]
    pub fn to_ocaml_string(&self) -> String {
        match self {
            Self::Unit => "unit".to_string(),
            Self::Ident(ident) => match ident.to_string().as_str() {
                "isize" => "int".to_string(),
                "usize" => "int".to_string(),
                "i32" => "Int32.t".to_string(),
                "i64" => "Int64.t".to_string(),
                "f32" | "f64" => "float".to_string(),
                "u8" => "char".to_string(),
                ident => ocamlize(ident),
            },
            Self::Tuple(tuple) => {
                let v: Vec<_> = tuple.iter().map(|x| x.to_ocaml_string()).collect();
                format!("({})", v.join(" * "))
            }
            Self::Option(ty) => {
                format!("{} option", ty.to_ocaml_string())
            }
            Self::VecArray(ty) => {
                format!("{} array", ty.to_ocaml_string())
            }
            Self::VecList(ty) => {
                format!("{} list", ty.to_ocaml_string())
            }
            Self::RustResult(ty) => {
                format!("({}, string) Result.t", ty.to_ocaml_string())
            }
            Self::BigArray1(ty) => {
                let (ocaml_type, elt_type) = match ty.as_ref() {
                    Self::Ident(ident) => match ident.to_string().as_str() {
                        "f64" => ("float".to_string(), "Bigarray.float64_elt".to_string()),
                        "f32" => ("float".to_string(), "Bigarray.float32_elt".to_string()),
                        "i64" => ("int".to_string(), "Bigarray.int64_elt".to_string()),
                        "i32" => ("int".to_string(), "Bigarray.int32_elt".to_string()),
                        "u8" => ("char".to_string(), "Bigarray.int8_unsigned_elt".to_string()),
                        ident => (ocamlize(ident), ocamlize(ident)),
                    },
                    _ => panic!("unexpected type nested in bigarray {:?}", self),
                };
                format!("({}, {}, string) Bigarray.Array1.t", ocaml_type, elt_type)
            }
            Self::Result(ty_ok, ty_err) => {
                format!("({}, {}) Result.t", ty_ok.to_ocaml_string(), ty_err.to_ocaml_string())
            }
            Self::Fn0(ty) => {
                format!("(unit -> ({}))", ty.to_ocaml_string())
            }
            Self::Fn1(ty_arg, ty_res) => {
                format!("(({}) -> ({}))", ty_arg.to_ocaml_string(), ty_res.to_ocaml_string())
            }
        }
    }
}

pub enum Lang {
    OCaml,
    Rust,
}

impl Lang {
    fn of_abi(abi: &Abi) -> Result<Self> {
        match &abi.name {
            None => Err(Error::new(abi.span(), "no abi name provided")),
            Some(name) => match name.value().as_str() {
                "OCaml" => Ok(Self::OCaml),
                "Rust" => Ok(Self::Rust),
                name => Err(Error::new(abi.span(), format!("unsupported abi name {}", name))),
            },
        }
    }
}

pub enum ModItem {
    Fn {
        ident: proc_macro2::Ident,
        args: Vec<(syn::PatIdent, Box<syn::Type>, Type)>,
        output: (Box<syn::Type>, Type),
    },
}

pub enum ApiItem {
    ForeignMod { attrs: Vec<Attribute>, lang: Lang, brace_token: token::Brace, items: Vec<ModItem> },
    Enum(syn::ItemEnum),
    Struct(syn::ItemStruct),
    Type(syn::ItemType),
    Include(String),
    Other(syn::Item),
}

pub struct Api {
    pub ident: Ident,
    pub api_items: Vec<ApiItem>,
}

impl Parse for Api {
    fn parse(input: ParseStream) -> Result<Self> {
        let _mod_token: Token![mod] = input.parse()?;
        let ident: Ident = input.parse()?;
        let content;
        let _brace_token = braced!(content in input);
        let mut api_items = Vec::new();
        while !content.is_empty() {
            api_items.push(content.parse()?);
        }
        Ok(Api { ident, api_items })
    }
}

impl Parse for ApiItem {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;

        let item = input.parse()?;
        match item {
            syn::Item::Macro(f) if f.mac.path.is_ident("ocaml_include") => {
                let f: syn::LitStr = f.mac.parse_body()?;
                Ok(ApiItem::Include(f.value()))
            }
            syn::Item::Struct(mut item) => {
                item.attrs.splice(..0, attrs);
                Ok(ApiItem::Struct(item.clone()))
            }
            syn::Item::Enum(mut item) => {
                item.attrs.splice(..0, attrs);
                Ok(ApiItem::Enum(item.clone()))
            }
            syn::Item::Type(mut item) => {
                item.attrs.splice(..0, attrs);
                Ok(ApiItem::Type(item.clone()))
            }
            syn::Item::ForeignMod(mut item) => {
                item.attrs.splice(..0, attrs);
                let mut mod_items = vec![];
                for item in item.items.into_iter() {
                    match item {
                        syn::ForeignItem::Fn(f) => {
                            let mut args = vec![];
                            for arg in f.sig.inputs.pairs() {
                                let (arg, _comma) = arg.into_tuple();
                                match arg {
                                    syn::FnArg::Typed(typed) => match &*typed.pat {
                                        syn::Pat::Ident(ident) => {
                                            let ty = Type::parse_type(&typed.ty)?;
                                            args.push((ident.clone(), typed.ty.clone(), ty))
                                        }
                                        _ => {
                                            return Err(Error::new(
                                                typed.span(),
                                                "only identifiers are supported",
                                            ));
                                        }
                                    },
                                    syn::FnArg::Receiver(_) => {
                                        return Err(Error::new(
                                            arg.span(),
                                            "self is not supported",
                                        ));
                                    }
                                }
                            }
                            let output = match &f.sig.output {
                                syn::ReturnType::Default => {
                                    (Box::new(syn::Type::Verbatim(quote! { () })), Type::Unit)
                                }
                                syn::ReturnType::Type(_arrow, type_) => {
                                    let ty = Type::parse_type(type_)?;
                                    (type_.clone(), ty)
                                }
                            };
                            mod_items.push(ModItem::Fn { ident: f.sig.ident, args, output })
                        }
                        _ => {
                            return Err(Error::new(item.span(), "unsupported in extern mod"));
                        }
                    }
                }
                Ok(ApiItem::ForeignMod {
                    attrs: item.attrs,
                    lang: Lang::of_abi(&item.abi)?,
                    brace_token: item.brace_token,
                    items: mod_items,
                })
            }
            other => Ok(ApiItem::Other(other)),
        }
    }
}

pub fn attr_is_ocaml_deriving(attr: &Attribute) -> bool {
    !attr.path.segments.is_empty() && attr.path.segments[0].ident == "ocaml_deriving"
}
