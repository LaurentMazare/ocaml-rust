use super::api::{attr_is_ocaml_deriving, Api, ApiItem, ModItem};
use quote::{quote, ToTokens};
use std::collections::BTreeSet;
use syn::parse::Error;
use syn::spanned::Spanned;

fn expand_enum(item: &syn::ItemEnum, expanded: &mut proc_macro2::TokenStream) -> syn::Result<()> {
    let mut item = item.clone();
    item.attrs = item.attrs.into_iter().filter(|x| !attr_is_ocaml_deriving(x)).collect();
    expanded.extend((&item).into_token_stream());
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
            impl ocaml_rust::from_value::NotF64 for #enum_ident {}
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
                        Self::#variant_ident => unsafe { ocaml_sys::val_int(#current_index) }
                    }
                }
                syn::Fields::Named(n) => {
                    let current_index = nonconst_index;
                    nonconst_index += 1;
                    let nfields = n.named.len();
                    let mut fields: Vec<proc_macro2::TokenStream> = Vec::new();
                    let mut set_fields: Vec<proc_macro2::TokenStream> = Vec::new();
                    for (field_idx, field) in n.named.iter().enumerate() {
                        let tmp_ident = syn::Ident::new(&format!("_tmp{}", field_idx), n.span());
                        let field_ident = &field.ident;
                        let ty = &field.ty;
                        fields.push(quote! { #field_ident });
                        set_fields.push(quote! {
                            let #tmp_ident = <#ty as ocaml_rust::to_value::ToValue>::to_value(#field_ident);
                            unsafe { ocaml_sys::store_field(rv.value().value, #field_idx, #tmp_ident)};
                        })
                    }
                    quote! {
                        Self::#variant_ident{#(#fields,)*} => {
                            let v = unsafe { ocaml_sys::caml_alloc(#nfields, #current_index) };
                            let rv : ocaml_rust::RootedValue<()> = ocaml_rust::RootedValue::create(v);
                            #(#set_fields)*
                            rv.value().value
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
                        let tmp_ident = syn::Ident::new(&format!("_tmp{}", field_idx), u.span());
                        let field_ident =
                            syn::Ident::new(&format!("_field{}", field_idx), u.span());
                        let ty = &field.ty;
                        fields.push(quote! { #field_ident });
                        set_fields.push(quote! {
                            let #tmp_ident = <#ty as ocaml_rust::to_value::ToValue>::to_value(#field_ident);
                            unsafe { ocaml_sys::store_field(rv.value().value, #field_idx,  #tmp_ident)};
                        })
                    }
                    quote! {
                        Self::#variant_ident(#(#fields,)*) => {
                            let v = unsafe { ocaml_sys::caml_alloc(#nfields, #current_index) };
                            let rv : ocaml_rust::RootedValue<()> = ocaml_rust::RootedValue::create(v);
                            #(#set_fields)*
                            rv.value().value
                        }
                    }
                }
            };
            variants.push(branch);
        }

        expanded.extend(quote! {
            impl ocaml_rust::to_value::ToValue for #enum_ident {
                fn to_value(&self) -> ocaml_sys::Value
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

fn expand_struct(
    item: &syn::ItemStruct,
    expanded: &mut proc_macro2::TokenStream,
) -> syn::Result<()> {
    let mut item = item.clone();
    item.attrs = item.attrs.into_iter().filter(|x| !attr_is_ocaml_deriving(x)).collect();
    expanded.extend((&item).into_token_stream());
    let struct_ident = &item.ident;
    let nfields = item.fields.len();
    let all_float = item.fields.iter().all(|field| match &field.ty {
        syn::Type::Path(path) => {
            let ty = &path.path.segments.last().unwrap().ident;
            ty == "f32" || ty == "f64"
        }
        _ => false,
    });
    {
        let mut fields: Vec<proc_macro2::TokenStream> = Vec::new();
        let mut let_fields: Vec<proc_macro2::TokenStream> = Vec::new();
        for (field_idx, field) in item.fields.iter().enumerate() {
            let field_ident = &field.ident;
            let ty = &field.ty;
            fields.push(quote! { #field_ident });
            let_fields.push(quote! {
                let _tmp_value = ocaml_sys::field(v, #field_idx);
                let #field_ident =
                <#ty as ocaml_rust::from_value::FromSysValue>::from_value(*_tmp_value);
            })
        }

        if all_float {
            let mut let_fields_float: Vec<proc_macro2::TokenStream> = Vec::new();
            for (field_idx, field) in item.fields.iter().enumerate() {
                let field_ident = &field.ident;
                let ty = &field.ty;
                let_fields_float.push(quote! {
                    let _tmp_value = ocaml_sys::field(v, #field_idx);
                    let #field_ident = *(_tmp_value as *const f64) as #ty;
                })
            }

            expanded.extend(quote! {
                impl ocaml_rust::from_value::FromSysValue for #struct_ident {
                    unsafe fn from_value(v: ocaml_sys::Value) -> Self {
                        let tag = ocaml_sys::tag_val(v);
                        if tag == ocaml_sys::DOUBLE_ARRAY {
                            #(#let_fields_float)*
                            #struct_ident { #(#fields,)* }
                        } else {
                            ocaml_rust::from_value::check_tag("record", v, 0);
                            #(#let_fields)*
                            #struct_ident { #(#fields,)* }
                        }
                    }
                }
            });
        } else {
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
    }
    {
        let mut fields: Vec<proc_macro2::TokenStream> = Vec::new();
        let mut set_fields: Vec<proc_macro2::TokenStream> = Vec::new();
        for (field_idx, field) in item.fields.iter().enumerate() {
            let tmp_ident = syn::Ident::new(&format!("_tmp{}", field_idx), item.span());
            let field_ident = &field.ident;
            let ty = &field.ty;
            fields.push(quote! { #field_ident });
            let q = if all_float {
                quote! {
                    let #tmp_ident = unsafe { ocaml_sys::field(v, #field_idx) as *mut f64 };
                    unsafe { *#tmp_ident = *#field_ident as f64 };
                }
            } else {
                quote! {
                    let #tmp_ident = <#ty as ocaml_rust::to_value::ToValue>::to_value(#field_ident);
                    unsafe { ocaml_sys::store_field(rv.value().value, #field_idx, #tmp_ident)};
                }
            };
            set_fields.push(q)
        }

        let init = if all_float {
            quote! { ocaml_sys::caml_alloc(#nfields, ocaml_sys::DOUBLE_ARRAY) }
        } else {
            quote! { ocaml_sys::caml_alloc_tuple(#nfields) }
        };

        expanded.extend(quote! {
            impl ocaml_rust::from_value::NotF64 for #struct_ident {}
            impl ocaml_rust::to_value::ToValue for #struct_ident {
                fn to_value(&self) -> ocaml_sys::Value
                {
                    let #struct_ident { #(#fields,)* } = self;
                    let v = unsafe { #init };
                    let rv : ocaml_rust::RootedValue<()> = ocaml_rust::RootedValue::create(v);
                    #(#set_fields)*
                    rv.value().value
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
    pub fn c_fn_name(&self, ident: &proc_macro2::Ident, namespace: Option<&Vec<String>>) -> String {
        let api_ident = &self.ident;
        let namespace = namespace.map_or_else(|| "".to_string(), |s| s.join("_") + "_");
        format!("__ocaml_{}{}_{}", api_ident, namespace, ident)
    }

    #[allow(dead_code)]
    pub fn expand(&self) -> syn::Result<proc_macro2::TokenStream> {
        let mut expanded = proc_macro2::TokenStream::new();
        for item in self.api_items.iter() {
            match item {
                ApiItem::ForeignMod { attrs: _, lang: _, brace_token: _, items } => {
                    for item in items.iter() {
                        match item {
                            ModItem::Fn { ident, args, output: (output, _), attrs } => {
                                let ocaml_ident = syn::Ident::new(
                                    &self.c_fn_name(ident, attrs.namespace.as_ref()),
                                    ident.span(),
                                );
                                let arg_with_types: Vec<_> = args
                                    .iter()
                                    .map(|(ident, _ty, _ty2)| quote! { #ident: ocaml_sys::Value})
                                    .collect();
                                let args_conv: Vec<_> =
                                    args.iter().map(|(ident, ty, _typ)| {
                                            let ty = match ty.as_ref() {
                                                syn::Type::Reference(ty) => ty.elem.as_ref(),
                                                other => other,
                                            };
                                        quote! {
                                        let mut #ident = unsafe {
                                            <#ty as ocaml_rust::from_value::FromSysValue>::from_value(#ident) };
                                        }}).collect();
                                let args: Vec<_> = args
                                    .iter()
                                    .map(|(ident, ty, _typ)| {
                                        if is_ref(ty.as_ref()) {
                                            quote! { &mut #ident }
                                        } else {
                                            quote! { #ident }
                                        }
                                    })
                                    .collect();
                                let namespace_ident = match &attrs.namespace {
                                    None => quote! { #ident},
                                    Some(namespace) => {
                                        let namespace = namespace
                                            .iter()
                                            .map(|s| syn::Ident::new(s, ident.span()));
                                        quote! { #(#namespace)::*::#ident }
                                    }
                                };
                                expanded.extend(quote! {
                                #[no_mangle]
                                pub extern "C" fn #ocaml_ident(#(#arg_with_types),*) -> ocaml_sys::Value {
                                    ocaml_rust::initial_setup();
                                    #(#args_conv)*;
                                    #[allow(clippy::unnecessary_mut_passed)]
                                    let res: #output = #namespace_ident(#(#args),*);
                                    <#output as ocaml_rust::to_value::ToValue>::to_value(&res)
                                } })
                            }
                        }
                    }
                }
                ApiItem::Enum(item) => expand_enum(item, &mut expanded)?,
                ApiItem::Struct(item) => expand_struct(item, &mut expanded)?,
                ApiItem::Type(item) => expand_type(item, &mut expanded)?,
                ApiItem::Include(_) => {}
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
                | ApiItem::Include(_)
                | ApiItem::Enum(_)
                | ApiItem::Struct(_)
                | ApiItem::Other(_) => None,
            })
            .collect()
    }
}
