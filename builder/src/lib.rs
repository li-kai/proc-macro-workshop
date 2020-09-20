extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

macro_rules! span_compile_error {
    ($ident:expr, $msg:expr) => {
        syn::Error::new($ident.span(), $msg)
            .to_compile_error()
            .into()
    };
}

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // eprintln!("INPUT: {:#?}", input.clone());

    let indent = &input.ident;
    let builder_indent = format_ident!("{}Builder", indent);

    let fields = &match input.data {
        syn::Data::Struct(data_struct) => {
            let struct_fields = data_struct.fields;
            match struct_fields {
                syn::Fields::Named(named_fields) => named_fields.named,
                _ => return span_compile_error!(input.ident, "expected named fields"),
            }
        }
        _ => return span_compile_error!(input.ident, "expected struct"),
    };

    let optional_fields = fields.iter().map(|field| {
        let name = &field.ident;
        let field_type = &field.ty;
        quote! {
            #name: std::option::Option<#field_type>
        }
    });

    let builder_values = fields.iter().map(|field| {
        let name = &field.ident;
        quote! {
            #name: None
        }
    });

    let builder_field_setters = fields.iter().map(|field| {
        let name = &field.ident;
        let field_type = &field.ty;
        quote! {
            fn #name(&mut self, #name: #field_type) -> &mut Self {
                self.#name = std::option::Option::Some(#name);
                self
            }
        }
    });

    let builder_field_creation = fields.iter().map(|field| {
        let name = &field
            .ident
            .as_ref()
            .expect("named fields do not have identifier");
        let error_msg = format!("field \"{}\" is None", name);
        quote! {
            #name: self.#name.clone().ok_or(#error_msg)?
        }
    });

    let expanded = quote! {
        pub struct #builder_indent {
            #(#optional_fields),*
        }

        impl #indent {
            pub fn builder() -> #builder_indent {
                #builder_indent {
                    #(#builder_values),*
                }
            }
        }

        impl #builder_indent {
            #(#builder_field_setters)*

            pub fn build(&mut self) -> std::result::Result<#indent, std::boxed::Box<dyn std::error::Error>> {
                Ok(#indent {
                    #(#builder_field_creation),*
                })
            }
        }
    };
    // eprintln!("TOKENS: {}", expanded);

    expanded.into()
}
