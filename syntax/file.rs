use crate::syntax::api::Api;
use syn::parse::discouraged::Speculative;
use syn::parse::{Error, Parse, ParseStream, Result};
use syn::{braced, Attribute, Ident, Item, Token, Visibility};

pub struct File {
    pub apis: Vec<Api>,
}

impl Parse for File {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut apis: Vec<Api> = Vec::new();
        input.call(Attribute::parse_inner)?;
        parse(input, &mut apis)?;
        Ok(File { apis })
    }
}

fn parse(input: ParseStream, apis: &mut Vec<Api>) -> Result<()> {
    while !input.is_empty() {
        let mut ocaml_rust = false;
        let attrs = input.call(Attribute::parse_outer)?;
        for attr in &attrs {
            let path = &attr.path.segments;
            if path.len() == 2 && path[0].ident == "ocaml_rust" && path[1].ident == "bridge" {
                ocaml_rust = true;
                break;
            }
        }

        let ahead = input.fork();
        ahead.parse::<Visibility>()?;
        ahead.parse::<Option<Token![unsafe]>>()?;
        if !ahead.peek(Token![mod]) {
            let item: Item = input.parse()?;
            if ocaml_rust {
                return Err(Error::new_spanned(item, "expected a module"));
            }
            continue;
        }

        if ocaml_rust {
            let api: Api = input.parse()?;
            apis.push(api);
        } else {
            input.advance_to(&ahead);
            input.parse::<Token![mod]>()?;
            input.parse::<Ident>()?;
            let semi: Option<Token![;]> = input.parse()?;
            if semi.is_none() {
                let content;
                braced!(content in input);
                parse(&content, apis)?;
            }
        }
    }
    Ok(())
}
