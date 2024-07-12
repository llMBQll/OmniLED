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

                    let initializer = match attrs.default {
                        Some(default) => quote!{
                            #initializer.unwrap_or(#default)
                        },
                        None => quote! {
                            #initializer.map_err(|err| {
                                err.with_context(|_| {
                                    format!("Error occurred when parsing {}.{}", stringify!(#name), stringify!(#field))
                                })
                            })?
                        },
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
            let idents: Vec<_> = data
                .variants
                .iter()
                .map(|variant| {
                    match &variant.fields {
                        syn::Fields::Named(_) => unimplemented!(),
                        syn::Fields::Unnamed(_) => todo!(),
                        syn::Fields::Unit => {}
                    }
                    &variant.ident
                })
                .collect();

            let names = idents.iter().map(|ident| {
                quote! {
                    stringify!(#ident),
                }
            });
            let names = quote! { vec![#(#names)*] };

            let initializers = idents.iter().map(|ident| {
                quote! {
                    stringify!(#ident) => Ok(Self::#ident),
                }
            });
            let initializers = quote! { #(#initializers)* };

            let initializer = quote! {
                mlua::Value::String(string) => {
                    match string.to_str().unwrap() {
                        #initializers
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
