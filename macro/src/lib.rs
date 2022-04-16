extern crate proc_macro;
mod syntax;
use proc_macro::TokenStream;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn bridge(_args: TokenStream, input: TokenStream) -> TokenStream {
    let api = parse_macro_input!(input as syntax::api::Api);
    api.expand().unwrap_or_else(|err| err.to_compile_error()).into()
}
