extern crate proc_macro;

use proc_macro::TokenStream;
use syn::parse_macro_input;

#[derive(Debug)]
struct IterateInput {
    keyword: syn::Ident,
    start_inclusive: syn::LitInt,
    end_exclusive: syn::LitInt,
}

impl syn::parse::Parse for IterateInput {
    fn parse(input: syn::parse::ParseStream) -> Result<Self, syn::Error> {
        let keyword = input.parse()?;
        input.parse::<syn::Token![in]>()?;
        let start_inclusive = input.parse()?;
        input.parse::<syn::Token![..]>()?;
        let end_exclusive = input.parse()?;

        let _;
        syn::braced!(_ in input);

        Ok(IterateInput {
            keyword,
            start_inclusive,
            end_exclusive,
        })
    }
}

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as IterateInput);

    eprintln!("INPUT: {:#?}", input);

    unimplemented!()
}
