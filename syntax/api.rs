// TODO: Support adding tags to the shared definition that would be added
// on the ocaml side, e.g. to add [@@deriving sexp].
use quote::{quote, ToTokens};
use std::collections::BTreeSet;
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

pub enum Type {
    Unit,
    Ident(proc_macro2::Ident),
    Tuple(Vec<Type>),
    VecArray(Box<Type>),
    VecList(Box<Type>),
    Option(Box<Type>),
    Result(Box<Type>, Box<Type>),
    Fn0(Box<Type>),
    Fn1(Box<Type>, Box<Type>),
}

impl Type {
    fn is_abstract(&self, abstract_types: &BTreeSet<proc_macro2::Ident>) -> bool {
        match self {
            Type::Ident(ident) => abstract_types.contains(ident),
            _ => false,
        }
    }

    pub fn parse_type(ty: &syn::Type) -> Result<Type> {
        match ty {
            syn::Type::Path(ty) => {
                let path = &ty.path;
                if ty.qself.is_none() && path.leading_colon.is_none() && path.segments.len() == 1 {
                    let segment = &path.segments[0];
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
                                if ident == "Fn0" {
                                    return Ok(Type::Fn0(Box::new(ty)));
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
                "i64" => "Int64.t".to_string(),
                "f64" => "float".to_string(),
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
                name => Err(Error::new(abi.span(), format!("unsupported abi name {name}"))),
            },
        }
    }
}

pub enum ModItem {
    Fn { ident: proc_macro2::Ident, args: Vec<(syn::PatIdent, Box<syn::Type>, Type)>, output: Type },
}

pub enum ApiItem {
    ForeignMod { attrs: Vec<Attribute>, lang: Lang, brace_token: token::Brace, items: Vec<ModItem> },
    Enum(syn::ItemEnum),
    Struct(syn::ItemStruct),
    Type(syn::ItemType),
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
                            let output = match f.sig.output {
                                syn::ReturnType::Default => Type::Unit,
                                syn::ReturnType::Type(_arrow, type_) => Type::parse_type(&type_)?,
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

fn expand_enum(item: &syn::ItemEnum, expanded: &mut proc_macro2::TokenStream) -> syn::Result<()> {
    expanded.extend(item.into_token_stream());
    let enum_ident = &item.ident;

    // FromValue
    {
        let mut const_branches: Vec<proc_macro2::TokenStream> = Vec::new();
        let mut nonconst_branches: Vec<proc_macro2::TokenStream> = Vec::new();
        let mut const_index = 0isize;
        let mut nonconst_index = 0u8;
        for variant in item.variants.iter() {
            let variant_ident = &variant.ident;
            match &variant.fields {
                syn::Fields::Unit => {
                    let current_index = const_index;
                    const_branches.push(quote! {
                        #current_index => { Self::#variant_ident }
                    });
                    const_index += 1;
                }
                syn::Fields::Named(n) => {
                    let current_index = nonconst_index;
                    nonconst_index += 1;

                    let mut fields: Vec<proc_macro2::TokenStream> = Vec::new();
                    let mut let_fields: Vec<proc_macro2::TokenStream> = Vec::new();
                    for (field_idx, field) in n.named.iter().enumerate() {
                        let field_ident = &field.ident;
                        let ty = &field.ty;
                        fields.push(quote! { #field_ident });
                        let_fields.push(quote! {
                            let _tmp_value = ocaml_sys::field(v, #field_idx);
                            let #field_ident =
                            <#ty as ocaml_rust::from_value::FromSysValue>::from_value(*_tmp_value);
                        })
                    }

                    nonconst_branches.push(quote! {
                        #current_index => {
                            #(#let_fields)*
                            Self::#variant_ident { #(#fields,)* }
                        }
                    })
                }
                syn::Fields::Unnamed(u) => {
                    let current_index = nonconst_index;
                    nonconst_index += 1;

                    let mut fields: Vec<proc_macro2::TokenStream> = Vec::new();
                    let mut let_fields: Vec<proc_macro2::TokenStream> = Vec::new();
                    for (field_idx, field) in u.unnamed.iter().enumerate() {
                        let field_ident =
                            syn::Ident::new(&format!("_field{}", field_idx), u.span());
                        let ty = &field.ty;
                        fields.push(quote! { #field_ident });
                        let_fields.push(quote! {
                            let _tmp_value = ocaml_sys::field(v, #field_idx);
                            let #field_ident =
                            <#ty as ocaml_rust::from_value::FromSysValue>::from_value(*_tmp_value);
                        })
                    }

                    nonconst_branches.push(quote! {
                        #current_index => {
                            #(#let_fields)*
                            Self::#variant_ident(#(#fields,)*)
                        }
                    })
                }
            }
        }
        expanded.extend(quote! {
            impl ocaml_rust::from_value::FromSysValue for #enum_ident {
                unsafe fn from_value(v: ocaml_sys::Value) -> Self {
                    if ocaml_sys::is_long(v) {
                        match ocaml_sys::int_val(v) {
                            #(#const_branches),*
                            tag => panic!("unexpected const tag {}", tag),
                        }
                    } else {
                        match ocaml_sys::tag_val(v) {
                            #(#nonconst_branches),*
                            tag => panic!("unexpected nonconst tag {}", tag)
                        }
                    }
                }
            }
        });
    }

    // ToValue
    {
        let mut variants: Vec<proc_macro2::TokenStream> = Vec::new();
        let mut const_index = 0isize;
        let mut nonconst_index = 0u8;
        for variant in item.variants.iter() {
            let variant_ident = &variant.ident;
            let branch = match &variant.fields {
                syn::Fields::Unit => {
                    let current_index = const_index;
                    const_index += 1;
                    quote! {
                        Self::#variant_ident => pin(unsafe { ocaml_sys::val_int(#current_index) })
                    }
                }
                syn::Fields::Named(n) => {
                    let current_index = nonconst_index;
                    nonconst_index += 1;
                    let nfields = n.named.len();
                    let mut fields: Vec<proc_macro2::TokenStream> = Vec::new();
                    let mut set_fields: Vec<proc_macro2::TokenStream> = Vec::new();
                    for (field_idx, field) in n.named.iter().enumerate() {
                        let field_ident = &field.ident;
                        let ty = &field.ty;
                        fields.push(quote! { #field_ident });
                        set_fields.push(quote! {
                            <#ty as ocaml_rust::to_value::ToValue>::to_value(
                                #field_ident,
                                |x| unsafe { ocaml_sys::store_field(v, #field_idx, x) },
                            );
                        })
                    }
                    quote! {
                        Self::#variant_ident{#(#fields,)*} => {
                            let v = unsafe { ocaml_sys::caml_alloc(#nfields, #current_index) };
                            let res = pin(v);
                            #(#set_fields)*
                            res
                        }
                    }
                }
                syn::Fields::Unnamed(u) => {
                    let current_index = nonconst_index;
                    nonconst_index += 1;
                    let nfields = u.unnamed.len();

                    let mut fields: Vec<proc_macro2::TokenStream> = Vec::new();
                    let mut set_fields: Vec<proc_macro2::TokenStream> = Vec::new();
                    for (field_idx, field) in u.unnamed.iter().enumerate() {
                        let field_ident =
                            syn::Ident::new(&format!("_field{}", field_idx), u.span());
                        let ty = &field.ty;
                        fields.push(quote! { #field_ident });
                        set_fields.push(quote! {
                            <#ty as ocaml_rust::to_value::ToValue>::to_value(
                                #field_ident,
                                |x| unsafe { ocaml_sys::store_field(v, #field_idx, x) },
                            );
                        })
                    }
                    quote! {
                        Self::#variant_ident(#(#fields,)*) => {
                            let v = unsafe { ocaml_sys::caml_alloc(#nfields, #current_index) };
                            let res = pin(v);
                            #(#set_fields)*
                            res
                        }
                    }
                }
            };
            variants.push(branch);
        }

        expanded.extend(quote! {
            impl ocaml_rust::to_value::ToValue for #enum_ident {
                fn to_value<F, U>(&self, pin: F) -> U
                where
                    U: Sized,
                    F: FnOnce(ocaml_sys::Value) -> U,
                {
                    match self {
                        #(#variants),*
                    }
                }
            }
        })
    }
    Ok(())
}

pub fn attr_is_ocaml_deriving(attr: &Attribute) -> bool {
    !attr.path.segments.is_empty() && attr.path.segments[0].ident == "ocaml_deriving"
}

fn expand_struct(
    item: &syn::ItemStruct,
    expanded: &mut proc_macro2::TokenStream,
) -> syn::Result<()> {
    let mut item = item.clone();
    item.attrs = item.attrs.into_iter().filter(|x| !attr_is_ocaml_deriving(x)).collect();
    expanded.extend((&item).into_token_stream());
    let struct_ident = &item.ident;
    let nfields = item.fields.len();
    {
        let mut fields: Vec<proc_macro2::TokenStream> = Vec::new();
        let mut let_fields: Vec<proc_macro2::TokenStream> = Vec::new();
        for (field_idx, field) in item.fields.iter().enumerate() {
            let field_ident = &field.ident;
            let ty = &field.ty;
            // TODO: This won't work for custom blocks used by abstract types.  It would be nicer to
            // handle custom blocks via a trait implementation rather than specific code in the macro
            fields.push(quote! { #field_ident });
            let_fields.push(quote! {
                let _tmp_value = ocaml_sys::field(v, #field_idx);
                let #field_ident =
                <#ty as ocaml_rust::from_value::FromSysValue>::from_value(*_tmp_value);
            })
        }

        // TODO: handle FLOAT_ARRAY
        expanded.extend(quote! {
            impl ocaml_rust::from_value::FromSysValue for #struct_ident {
                unsafe fn from_value(v: ocaml_sys::Value) -> Self {
                    ocaml_rust::from_value::check_tag("record", v, 0);
                    #(#let_fields)*
                    #struct_ident { #(#fields,)* }
                }
            }
        });
    }
    {
        let mut fields: Vec<proc_macro2::TokenStream> = Vec::new();
        let mut set_fields: Vec<proc_macro2::TokenStream> = Vec::new();
        for (field_idx, field) in item.fields.iter().enumerate() {
            let field_ident = &field.ident;
            let ty = &field.ty;
            fields.push(quote! { #field_ident });
            set_fields.push(quote! {
                <#ty as ocaml_rust::to_value::ToValue>::to_value(
                    #field_ident,
                    |x| unsafe { ocaml_sys::store_field(v, #field_idx, x) },
                );
            })
        }

        expanded.extend(quote! {
            impl ocaml_rust::to_value::ToValue for #struct_ident {
                fn to_value<F, U>(&self, pin: F) -> U
                where
                    U: Sized,
                    F: FnOnce(ocaml_sys::Value) -> U,
                {
                    let #struct_ident { #(#fields,)* } = self;
                    let v = unsafe { ocaml_sys::caml_alloc_tuple(#nfields) };
                    let res = pin(v);
                    #(#set_fields)*
                    res
                }
            }
        });
    }
    Ok(())
}

fn expand_type(item: &syn::ItemType, expanded: &mut proc_macro2::TokenStream) -> syn::Result<()> {
    expanded.extend(item.into_token_stream());
    Ok(())
}

fn is_ref(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Reference(_))
}

impl Api {
    #[allow(dead_code)]
    pub fn c_fn_name(&self, ident: &proc_macro2::Ident) -> String {
        let api_ident = &self.ident;
        format!("__ocaml_{api_ident}_{ident}")
    }

    #[allow(dead_code)]
    pub fn expand(&self) -> syn::Result<proc_macro2::TokenStream> {
        let abstract_types = self.abstract_types();
        let mut expanded = proc_macro2::TokenStream::new();
        for item in self.api_items.iter() {
            match item {
                ApiItem::ForeignMod { attrs: _, lang: _, brace_token: _, items } => {
                    for item in items.iter() {
                        match item {
                            ModItem::Fn { ident, args, output } => {
                                let ocaml_ident =
                                    syn::Ident::new(&self.c_fn_name(ident), ident.span());
                                let arg_with_types: Vec<_> = args
                                    .iter()
                                    .map(|(ident, _ty, _ty2)| quote! { #ident: ocaml_sys::Value})
                                    .collect();
                                let args_conv: Vec<_> =
                                    args.iter().map(|(ident, ty, typ)| {
                                        if typ.is_abstract(&abstract_types) {
                                            let ty_ident = match typ {
                                                Type::Ident(ident) => ident,
                                                _ => panic!("must be ident"),
                                            };
                                            quote! {
                                                let #ident: ocaml_rust::Value<Box<#ty_ident>> = unsafe { ocaml_rust::Value::new(#ident) };
                                                let mut #ident = unsafe { &mut *ocaml_rust::custom::get(#ident).get() };
                                            }
                                        } else {
                                            let ty = match ty.as_ref() {
                                                syn::Type::Reference(ty) => ty.elem.as_ref(),
                                                other => other,
                                            };
                                        quote! {
                                        let mut #ident = unsafe {
                                            <#ty as ocaml_rust::from_value::FromSysValue>::from_value(#ident) };
                                        }}}).collect();
                                let args: Vec<_> = args
                                    .iter()
                                    .map(|(ident, ty, typ)| {
                                        if !typ.is_abstract(&abstract_types) && is_ref(ty.as_ref())
                                        {
                                            quote! { &mut #ident}
                                        } else {
                                            quote! { #ident }
                                        }
                                    })
                                    .collect();
                                let post_process_res = if output.is_abstract(&abstract_types) {
                                    quote! {
                                        ocaml_rust::gc::with_gc(|gc| ocaml_rust::custom::new(gc, res).value)
                                    }
                                } else {
                                    quote! {
                                        let rooted_res = ocaml_rust::to_value::to_rooted_value(&res);
                                        rooted_res.value().value
                                    }
                                };
                                expanded.extend(quote! {
                                #[no_mangle]
                                pub extern "C" fn #ocaml_ident(#(#arg_with_types),*) -> ocaml_sys::Value {
                                    ocaml_rust::initial_setup();
                                    #(#args_conv)*;
                                    let res = #ident(#(#args),*);
                                    #post_process_res
                                } })
                            }
                        }
                    }
                }
                ApiItem::Enum(item) => expand_enum(item, &mut expanded)?,
                ApiItem::Struct(item) => expand_struct(item, &mut expanded)?,
                ApiItem::Type(item) => expand_type(item, &mut expanded)?,
                ApiItem::Other(other) => {
                    return Err(Error::new(other.span(), "unsupported"));
                }
            }
        }
        Ok(expanded)
    }

    #[allow(dead_code)]
    pub fn abstract_types(&self) -> BTreeSet<proc_macro2::Ident> {
        self.api_items
            .iter()
            .filter_map(|api_item| match api_item {
                ApiItem::Type(item) => Some(item.ident.clone()),
                ApiItem::ForeignMod { .. }
                | ApiItem::Enum(_)
                | ApiItem::Struct(_)
                | ApiItem::Other(_) => None,
            })
            .collect()
    }
}
