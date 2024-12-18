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

use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Attribute, Data, DeriveInput};

use crate::common::{get_attribute, get_attribute_with_default_value, parse_attributes};

pub fn expand_lua_value_derive(input: DeriveInput) -> proc_macro::TokenStream {
    let name = input.ident;
    let (_impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let struct_attrs = get_struct_attributes(&input.attrs);
    let initializer = generate_initializer(&name, &input.data);

    let validate = match struct_attrs.validate {
        Some(validate) => quote! {
            match result {
                Ok(value) => match #validate(&value) {
                    Ok(_) => Ok(value),
                    Err(err) => Err(err),
                },
                Err(err) => Err(err),
            }
        },
        None => quote! { result },
    };

    let expanded = quote! {
        impl FromLua for #name #ty_generics {
            fn from_lua(value: mlua::Value, lua: &mlua::Lua) -> mlua::Result<#name #ty_generics #where_clause> {
                let result = match value {
                    #initializer,
                    mlua::Value::UserData(user_data) => {
                        let data = user_data.borrow::<#name>()?;
                        Ok(data.clone())
                    },
                    other => Err(mlua::Error::FromLuaConversionError {
                        from: other.type_name(),
                        to: String::from(stringify!(#name)),
                        message: None,
                    }),
                };
                #validate
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

fn generate_initializer(name: &Ident, data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => match data.fields {
            syn::Fields::Named(ref fields) => {
                let names = fields.named.iter().map(|f| {
                    let field = &f.ident;

                    let attrs = get_field_attributes(&f.attrs);

                    let initializer = quote! {
                        table.get(stringify!(#field))
                    };

                    let initializer = match attrs.transform {
                        Some(transform) => quote! {
                            #initializer.and_then(|x| #transform(x, lua))
                        },
                        None => initializer,
                    };

                    let initializer = quote! {
                        #initializer.map_err(|err| {
                            err.with_context(|_| {
                                format!("Error occurred when parsing {}.{}", stringify!(#name), stringify!(#field))
                            })
                        })?
                    };

                    let initializer = match attrs.default {
                        Some(default) => quote!{
                            {
                                if !table.contains_key(stringify!(#field))? {
                                    #default
                                } else {
                                    #initializer
                                }
                            }
                        },
                        None => initializer,
                    };

                    quote! {
                        #field: #initializer,
                    }
                });
                let initializer = quote! { #(#names)* };

                quote! {
                    mlua::Value::Table(table) => Ok(#name {
                        #initializer
                    })
                }
            }
            syn::Fields::Unnamed(_) | syn::Fields::Unit => unimplemented!(),
        },

        Data::Enum(ref data) => {
            let fields: Vec<_> = data
                .variants
                .iter()
                .map(|variant| match &variant.fields {
                    syn::Fields::Named(_) => unimplemented!(),
                    syn::Fields::Unnamed(_) => (FieldType::Unnamed, &variant.ident),
                    syn::Fields::Unit => (FieldType::Unit, &variant.ident),
                })
                .collect();

            let names = fields.iter().map(|field| {
                let ident = field.1;
                quote! {
                    stringify!(#ident),
                }
            });
            let names = quote! { vec![#(#names)*] };

            let mut unnamed_initializers = Vec::new();
            let mut unit_initializers = Vec::new();

            for field in fields {
                match field {
                    (FieldType::Unnamed, ident) => {
                        unnamed_initializers.push(quote! {
                            else if table.contains_key(stringify!(#ident))? {
                                Ok(Self::#ident(table.get(stringify!(#ident))?))
                            }
                        });
                    }
                    (FieldType::Unit, ident) => {
                        unit_initializers.push(quote! {
                            stringify!(#ident) => Ok(Self::#ident),
                        });
                    }
                }
            }

            let unnamed_initializers = quote! { #(#unnamed_initializers)* };
            let unit_initializers = quote! { #(#unit_initializers)* };

            let initializer = quote! {
                mlua::Value::Table(table) => {
                    if false {
                        unreachable!();
                    }
                    #unnamed_initializers
                    else {
                        Err(mlua::Error::runtime(format!(
                            "Expected one of {:?}",
                            #names
                        )))
                    }
                },
                mlua::Value::String(string) => {
                    match &*string.to_str().unwrap() {
                        #unit_initializers
                        string => Err(mlua::Error::runtime(format!(
                            "Expected one of {:?}, got '{}'",
                            #names,
                            string
                        ))),
                    }
                }
            };

            initializer
        }
        Data::Union(_) => unimplemented!(),
    }
}

#[derive(Debug)]
enum FieldType {
    // Named,
    Unnamed,
    Unit,
}

struct StructAttributes {
    validate: Option<TokenStream>,
}

fn get_struct_attributes(attributes: &Vec<Attribute>) -> StructAttributes {
    let mut attributes = parse_attributes("mlua", attributes);

    StructAttributes {
        validate: get_attribute(&mut attributes, "validate"),
    }
}

struct FieldAttributes {
    default: Option<TokenStream>,
    transform: Option<TokenStream>,
}

fn get_field_attributes(attributes: &Vec<Attribute>) -> FieldAttributes {
    let mut attributes = parse_attributes("mlua", attributes);

    FieldAttributes {
        default: get_attribute_with_default_value(
            &mut attributes,
            "default",
            quote!(Default::default()),
        ),
        transform: get_attribute(&mut attributes, "transform"),
    }
}
