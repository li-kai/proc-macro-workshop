extern crate proc_macro;

use proc_macro::TokenStream;
// use quote::{quote};
use syn::parse_macro_input;

#[derive(Debug)]
struct IterateInput {
    keyword: syn::Ident,
    start_inclusive: syn::LitInt,
    end_exclusive: syn::LitInt,
    content: TokenStream,
}

impl syn::parse::Parse for IterateInput {
    fn parse(input: syn::parse::ParseStream) -> Result<Self, syn::Error> {
        let keyword = input.parse()?;
        input.parse::<syn::Token![in]>()?;
        let start_inclusive = input.parse()?;
        input.parse::<syn::Token![..]>()?;
        let end_exclusive = input.parse()?;

        let content_parse_buffer;
        syn::braced!(content_parse_buffer in input);
        let content = content_parse_buffer.cursor().token_stream().into();

        Ok(IterateInput {
            keyword,
            start_inclusive,
            end_exclusive,
            content,
        })
    }
}

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as IterateInput);

    eprintln!("INPUT: {:#?}", input);

    // let expanded = quote! {};

    // eprintln!("TOKENS: {:#?}", expanded);

    // expanded.into()
    unimplemented!()
}
