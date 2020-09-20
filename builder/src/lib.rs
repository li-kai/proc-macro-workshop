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
        let field_type = if let Some(nested_field_type) = get_optional_field_type(&field.ty) {
            nested_field_type
        } else {
            &field.ty
        };

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
        let field_type = if let Some(nested_field_type) = get_optional_field_type(&field.ty) {
            nested_field_type
        } else {
            &field.ty
        };

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

        if let Some(_) = get_optional_field_type(&field.ty) {
            // as None is an acceptable value, we do not unwrap the Option
            return quote! {
                #name: self.#name.clone()
            };
        }
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

fn get_optional_field_type(field_type: &syn::Type) -> Option<&syn::Type> {
    let path_segments = match field_type {
        syn::Type::Path(syn::TypePath {
            qself: None,
            path:
                syn::Path {
                    leading_colon: None,
                    segments,
                },
        }) => segments,
        _ => return None,
    };
    if path_segments.is_empty() {
        return None;
    }
    let path_segment = &path_segments[0];

    if path_segment.ident != "Option" {
        return None;
    }
    let args = match &path_segment.arguments {
        syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
            colon2_token: None,
            lt_token: _,
            gt_token: _,
            args,
        }) => args,
        _ => return None,
    };

    if let Some(syn::GenericArgument::Type(sub_field_type)) = args.first() {
        Some(sub_field_type)
    } else {
        None
    }
}
