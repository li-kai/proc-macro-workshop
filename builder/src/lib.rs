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

#[allow(dead_code)]
struct ExpandToArg {
    keyword: syn::Ident,
    equal_token: syn::Token![=],
    name: syn::LitStr,
}

impl syn::parse::Parse for ExpandToArg {
    fn parse(input: syn::parse::ParseStream) -> Result<Self, syn::Error> {
        Ok(ExpandToArg {
            keyword: input.parse()?,
            equal_token: input.parse()?,
            name: input.parse()?,
        })
    }
}

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let ident = &input.ident;
    let builder_ident = format_ident!("{}Builder", ident);

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

        if get_vec_field_type(field_type).is_some() {
            quote! {
                #name: #field_type
            }
        } else {
            let normalized_field_type = get_optional_field_type(field_type).unwrap_or(field_type);

            quote! {
                #name: std::option::Option<#normalized_field_type>
            }
        }
    });

    let builder_values = fields.iter().map(|field| {
        let name = &field.ident;
        let field_type = &field.ty;

        if get_vec_field_type(field_type).is_some() {
            quote! {
                #name: std::vec::Vec::new()
            }
        } else {
            quote! {
                #name: None
            }
        }
    });

    let builder_field_setters = fields.iter().map(|field| {
        let name = &field.ident;
        let field_type = &field.ty;

        if let Some(vec_field_type) = get_vec_field_type(field_type) {
            let field_setter_fn = quote! {
                fn #name(&mut self, #name: #field_type) -> &mut Self {
                    self.#name = #name;
                    self
                }
            };

            let each_attr = match field
                .attrs
                .iter()
                .find(|attr| attr.path.is_ident("builder"))
            {
                Some(each_attr) => each_attr,
                None => return field_setter_fn,
            };
            let each_attr_name = match each_attr.parse_args::<ExpandToArg>() {
                Ok(expand) if expand.keyword == "each" => {
                    format_ident!("{}", expand.name.value().trim_matches('"'))
                }
                Ok(expand) => {
                    return span_compile_error!(
                        expand.keyword,
                        "expected `builder(each = \"...\")`"
                    );
                }
                _ => return field_setter_fn,
            };

            let each_field_setter_fn = quote! {
                fn #each_attr_name(&mut self, #each_attr_name: #vec_field_type) -> &mut Self {
                    self.#name.push(#each_attr_name);
                    self
                }
            };

            if &Some(each_attr_name) == name {
                each_field_setter_fn
            } else {
                quote! {
                    #field_setter_fn
                    #each_field_setter_fn
                }
            }
        } else {
            let normalized_field_type = get_optional_field_type(field_type).unwrap_or(field_type);

            quote! {
                fn #name(&mut self, #name: #normalized_field_type) -> &mut Self {
                    self.#name = std::option::Option::Some(#name);
                    self
                }
            }
        }
    });

    let builder_field_creation = fields.iter().map(|field| {
        let name = &field
            .ident
            .as_ref()
            .expect("named fields do not have identifier");
        let error_msg = format!("field \"{}\" is None", name);

        if get_optional_field_type(&field.ty).is_some() || get_vec_field_type(&field.ty).is_some() {
            // for optional fields, None is an acceptable value, we do not unwrap the Option
            // for vector fields, their default is an empty vector, so we just clone it
            quote! {
                #name: self.#name.clone()
            }
        } else {
            quote! {
                #name: self.#name.clone().ok_or(#error_msg)?
            }
        }
    });

    let expanded = quote! {
        pub struct #builder_ident {
            #(#optional_fields),*
        }

        impl #ident {
            pub fn builder() -> #builder_ident {
                #builder_ident {
                    #(#builder_values),*
                }
            }
        }

        impl #builder_ident {
            #(#builder_field_setters)*

            pub fn build(&mut self) -> std::result::Result<#ident, std::boxed::Box<dyn std::error::Error>> {
                Ok(#ident {
                    #(#builder_field_creation),*
                })
            }
        }
    };

    expanded.into()
}

fn get_field_type<'ty>(field_type: &'ty syn::Type, ident_name: &'_ str) -> Option<&'ty syn::Type> {
    let path_segments = match field_type {
        syn::Type::Path(syn::TypePath { qself: _, path }) => &path.segments,
        _ => return None,
    };
    let path_segment = match path_segments.first() {
        Some(segment) if segment.ident == ident_name => segment,
        _ => return None,
    };
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

fn get_optional_field_type(field_type: &syn::Type) -> Option<&syn::Type> {
    get_field_type(field_type, "Option")
}

fn get_vec_field_type(field_type: &syn::Type) -> Option<&syn::Type> {
    get_field_type(field_type, "Vec")
}
