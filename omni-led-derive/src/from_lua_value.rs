use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Attribute, Data, DeriveInput};

use crate::common::{get_attribute, get_attribute_with_default_value, is_option, parse_attributes};

pub fn expand_lua_value_derive(input: DeriveInput) -> proc_macro::TokenStream {
    let name = input.ident;
    let struct_attrs = get_struct_attributes(&input.attrs);
    let (initializer, handle_deprecated) = generate_initializer(&name, &input.data);

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
        impl FromLua for #name {
            fn from_lua(value: mlua::Value, lua: &mlua::Lua) -> mlua::Result<#name> {
                let result = match value {
                    #initializer,
                    mlua::Value::UserData(user_data) => {
                        let value = user_data.borrow::<#name>()?;
                        Ok(value.clone())
                    },
                    other => Err(mlua::Error::FromLuaConversionError {
                        from: other.type_name(),
                        to: String::from(stringify!(#name)),
                        message: None,
                    }),
                };
                #handle_deprecated
                #validate
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

fn generate_initializer(name: &Ident, data: &Data) -> (TokenStream, Option<TokenStream>) {
    match *data {
        Data::Struct(ref data) => match data.fields {
            syn::Fields::Named(ref fields) => {
                let mut deprecated_field_handlers = Vec::new();

                let names = fields.named.iter().map(|f| {
                    let field = &f.ident;

                    let attrs = get_field_attributes(&f.attrs);

                    attrs.deprecated.map(|target| {
                        deprecated_field_handlers.push(quote! {
                            match (value.#field.is_some(), value.#target.is_some()) {
                                (true, true) => {
                                    Err(mlua::Error::runtime(format!(
                                        "Both '{}' and '{}' are set, use '{}'",
                                        stringify!(#field),
                                        stringify!(#target),
                                        stringify!(#target)
                                    )))
                                }
                                (true, false) => {
                                    warn!(
                                        "'{}' field is deprecated, use '{}'",
                                        stringify!(#field),
                                        stringify!(#target)
                                    );
                                    std::mem::swap(&mut value.#field, &mut value.#target);
                                    Ok(())
                                }
                                (false, true) => {
                                    Ok(())
                                }
                                (false, false) => {
                                    Err(mlua::Error::runtime(
                                        "Key not found".to_string()
                                    ))
                                }
                            }.with_context(|_| {
                                format!(
                                    "Error occurred when parsing {}.{}",
                                    stringify!(#name),
                                    stringify!(#target)
                                )
                            })?;
                        });
                    });

                    let transform = match attrs.transform {
                        Some(transform) => quote! { #transform(x, lua) },
                        None => quote! { Ok(x) },
                    };

                    let default = match (attrs.default, is_option(&f.ty)) {
                        (Some(default), _) => quote! { Ok(#default) },
                        (None, true) => quote! { Ok(None) },
                        (None, false) => quote! { Err(mlua::Error::runtime("Key not found".to_string())) },
                    };

                    let initializer = quote! {
                        table.get::<Option<_>>(stringify!(#field))
                            .and_then(|optional| match optional {
                                Some(x) => #transform,
                                None => #default,
                            })
                            .with_context(|_| {
                                format!("Error occurred when parsing {}.{}", stringify!(#name), stringify!(#field))
                            })?
                    };

                    quote! {
                        #field: #initializer,
                    }
                });

                let initializer = quote! {
                    mlua::Value::Table(table) => Ok(#name {
                        #(#names)*
                    })
                };

                let handle_deprecated = if deprecated_field_handlers.is_empty() {
                    None
                } else {
                    let handlers = quote! { #(#deprecated_field_handlers)* };
                    Some(quote! {
                        let result = result.and_then(|mut value| {
                            #handlers
                            Ok(value)
                        });
                    })
                };

                (initializer, handle_deprecated)
            }
            syn::Fields::Unnamed(_) | syn::Fields::Unit => unimplemented!(),
        },

        Data::Enum(_) => unimplemented!(),
        Data::Union(_) => unimplemented!(),
    }
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
    deprecated: Option<TokenStream>,
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
        deprecated: get_attribute(&mut attributes, "deprecated"),
        transform: get_attribute(&mut attributes, "transform"),
    }
}
