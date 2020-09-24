extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use syn::parse_macro_input;

#[derive(Debug)]
struct IterateInput {
    keyword: syn::Ident,
    range: std::ops::Range<usize>,
    content: proc_macro2::TokenStream,
}

impl syn::parse::Parse for IterateInput {
    fn parse(input: syn::parse::ParseStream) -> Result<Self, syn::Error> {
        let keyword = input.parse()?;
        input.parse::<syn::Token![in]>()?;
        let start_inclusive = input.parse::<syn::LitInt>()?.base10_parse::<usize>()?;
        input.parse::<syn::Token![..]>()?;
        let end_exclusive = input.parse::<syn::LitInt>()?.base10_parse::<usize>()?;

        let content_parse_buffer;
        syn::braced!(content_parse_buffer in input);
        let content = content_parse_buffer.parse()?;

        Ok(IterateInput {
            keyword,
            range: (start_inclusive..end_exclusive),
            content,
        })
    }
}

fn replace_ident(
    input: proc_macro2::TokenStream,
    replacement_ident: &proc_macro2::Ident,
    replacement_literal: &proc_macro2::Literal,
) -> proc_macro2::TokenStream {
    // Return the remaining tokens, but replace identifiers.
    input
        .into_iter()
        .map(|tt| match tt {
            TokenTree::Group(g) => {
                let mut group = proc_macro2::Group::new(
                    g.delimiter(),
                    replace_ident(g.stream(), replacement_ident, replacement_literal),
                );
                group.set_span(g.span());
                proc_macro2::TokenTree::Group(group)
            }
            TokenTree::Punct(p) => {
                let mut punct = proc_macro2::Punct::new(p.as_char(), p.spacing());
                punct.set_span(p.span());
                proc_macro2::TokenTree::Punct(punct)
            }
            TokenTree::Ident(ident) => {
                if &ident == replacement_ident {
                    let mut literal = proc_macro2::TokenTree::Literal(replacement_literal.clone());
                    literal.set_span(ident.span());
                    literal
                } else {
                    proc_macro2::TokenTree::Ident(ident)
                }
            }
            tt => tt,
        })
        .collect()
}

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as IterateInput);

    // eprintln!("INPUT: {:#?}", input.content);

    let tokens = input
        .range
        .clone()
        .map(|i| {
            let literal = proc_macro2::Literal::usize_unsuffixed(i);
            let new_content = replace_ident(input.content.clone(), &input.keyword, &literal);
            new_content
        })
        .collect::<proc_macro2::TokenStream>();
    // eprintln!("tokens: {:#?}", tokens);

    tokens.into()
}
