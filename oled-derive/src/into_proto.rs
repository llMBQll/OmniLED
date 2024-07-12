use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Type};

use crate::common::{get_content, is_attribute};

pub fn expand_into_proto_derive(input: DeriveInput) -> proc_macro::TokenStream {
    let name = input.ident;
    // let (_impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let assignments = generate_assignments(&input.data);

    // TODO handle generics of deriving type
    let expanded = quote! {
        impl Into<oled_api::types::Table> for #name {
            fn into(self) -> oled_api::types::Table {
                let mut table = oled_api::types::Table::default();
                #assignments
                table
            }
        }
    };
    proc_macro::TokenStream::from(expanded)
}

fn generate_assignments(data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => match data.fields {
            syn::Fields::Named(ref fields) => {
                let assignments = fields.named.iter().map(|field| {
                    let field_identifier = &field.ident;
                    let field_name = match field_identifier {
                        Some(field) => format!("{}", field),
                        None => String::new(),
                    };
                    let renamed = field_name.to_case(Case::Pascal);

                    // TODO find a better way of checking whether type is an Option<_> or not
                    let is_option = if let Type::Path(type_path) = &field.ty {
                        type_path.path.segments[0].ident.to_string() == "Option"
                    } else {
                        false
                    };

                    let attrs = parse_attributes(&field.attrs);

                    let value_accessor = if is_option {
                        quote! { value }
                    } else {
                        quote! { self.#field_identifier }
                    };

                    let transformed = match attrs.transform {
                        Some(transform) => {
                            quote! { #transform(#value_accessor) }
                        }
                        None => quote! { #value_accessor },
                    };

                    let insertion = quote! {
                        table.items.insert(#renamed.to_string(), #transformed.into());
                    };

                    if is_option {
                        quote! {
                            if let Some(value) = self.#field_identifier {
                                #insertion
                            }
                        }
                    } else {
                        insertion
                    }
                });
                quote! { #(#assignments)* }
            }
            syn::Fields::Unnamed(_) | syn::Fields::Unit => unimplemented!(),
        },
        Data::Enum(_) | Data::Union(_) => unimplemented!(),
    }
}

struct ProtoAttributes {
    transform: Option<TokenStream>,
}

fn parse_attributes(attributes: &Vec<Attribute>) -> ProtoAttributes {
    let mut transform: Option<TokenStream> = None;

    for attr in attributes {
        if !attr.path().is_ident("proto") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if is_attribute(&meta, "transform") {
                transform = Some(get_content(&meta)?);
                return Ok(());
            }

            Ok(())
        })
        .unwrap();
    }

    ProtoAttributes { transform }
}
