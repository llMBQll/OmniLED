use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Attribute, Data, DeriveInput};

use crate::common::{get_content, get_optional_content, is_attribute};

pub fn expand_lua_value_derive(input: DeriveInput) -> proc_macro::TokenStream {
    let name = input.ident;
    let (_impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let initializer = generate_initializer(&name, &input.data);

    // TODO handle generics of deriving type, for now only lifetime "'a" is allowed
    let expanded = quote! {
        impl<'a> FromLua<'a> for #name #ty_generics {
            fn from_lua(value: mlua::Value<'a>, lua: &'a mlua::Lua) -> mlua::Result<#name #ty_generics #where_clause> {
                match value {
                    #initializer,
                    mlua::Value::UserData(user_data) => {
                        let data = user_data.borrow::<#name>()?;
                        Ok(data.clone())
                    },
                    other => Err(mlua::Error::FromLuaConversionError {
                        from: other.type_name(),
                        to: stringify!(#name),
                        message: None,
                    }),
                }
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

                    let attrs = parse_attributes(&f.attrs);

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
                    match string.to_str().unwrap() {
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

struct LuaAttributes {
    default: Option<TokenStream>,
    transform: Option<TokenStream>,
}

fn parse_attributes(attributes: &Vec<Attribute>) -> LuaAttributes {
    let mut default: Option<TokenStream> = None;
    let mut transform: Option<TokenStream> = None;

    for attr in attributes {
        if !attr.path().is_ident("mlua") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if is_attribute(&meta, "default") {
                default = match get_optional_content(&meta)? {
                    Some(content) => Some(content),
                    None => Some(quote!(Default::default())),
                };
                return Ok(());
            }

            if is_attribute(&meta, "transform") {
                transform = Some(get_content(&meta)?);
                return Ok(());
            }

            Ok(())
        })
        .unwrap();
    }

    LuaAttributes { default, transform }
}
