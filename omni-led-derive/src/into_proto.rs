/*
 * OmniLED is a software for displaying data on various OLED devices.
 * Copyright (C) 2024  Michał Bałabanow <m.balabanow@gmail.com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use convert_case::Casing;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Type};

use crate::common::{get_attribute, get_case, parse_attributes};

pub fn expand_into_proto_derive(input: DeriveInput) -> proc_macro::TokenStream {
    let name = input.ident;
    // let (_impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let struct_attrs = get_struct_attributes(&input.attrs);
    let assignments = generate_assignments(&input.data, &struct_attrs);

    // TODO handle generics of deriving type
    let expanded = quote! {
        impl Into<omni_led_api::types::Table> for #name {
            fn into(self) -> omni_led_api::types::Table {
                let mut table = omni_led_api::types::Table::default();
                #assignments
                table
            }
        }

        impl Into<omni_led_api::types::Field> for #name {
            fn into(self) -> omni_led_api::types::Field {
                let table = self.into();
                omni_led_api::types::Field {
                    field: Some(omni_led_api::types::field::Field::FTable(table)),
                }
            }
        }
    };
    proc_macro::TokenStream::from(expanded)
}

fn generate_assignments(data: &Data, struct_attrs: &StructAttributes) -> TokenStream {
    match *data {
        Data::Struct(ref data) => match data.fields {
            syn::Fields::Named(ref fields) => {
                let assignments = fields.named.iter().map(|field| {
                    let field_identifier = &field.ident;
                    let field_name = match field_identifier {
                        Some(field) => format!("{}", field),
                        None => String::new(),
                    };

                    let renamed = match &struct_attrs.rename_all {
                        Some(rename_all) => field_name.to_case(get_case(&rename_all)),
                        None => field_name,
                    };

                    // TODO find a better way of checking whether type is an Option<_> or not
                    let is_option = match &field.ty {
                        Type::Path(type_path) => {
                            type_path.path.segments[0].ident.to_string() == "Option"
                        }
                        _ => false,
                    };

                    let attrs = get_field_attributes(&field.attrs);

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

struct StructAttributes {
    rename_all: Option<TokenStream>,
}

fn get_struct_attributes(attributes: &Vec<Attribute>) -> StructAttributes {
    let mut attributes = parse_attributes("proto", attributes);

    StructAttributes {
        rename_all: get_attribute(&mut attributes, "rename_all"),
    }
}

struct FieldAttributes {
    transform: Option<TokenStream>,
}

fn get_field_attributes(attributes: &Vec<Attribute>) -> FieldAttributes {
    let mut attributes = parse_attributes("proto", attributes);

    FieldAttributes {
        transform: get_attribute(&mut attributes, "transform"),
    }
}
