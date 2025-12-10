use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Type};

use crate::common::{get_attribute, get_attribute_with_default_value, is_option, parse_attributes};

pub fn expand_lua_value_derive(input: DeriveInput) -> proc_macro::TokenStream {
    let name = input.ident;
    let struct_attrs = StructAttributes::parse(&input.attrs);
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
    let fields = get_fields(data);
    match fields {
        Fields::Struct(struct_fields) => {
            let mut deprecated_field_handlers = Vec::new();

            let names = struct_fields.into_iter().map(|f| {
                let attrs = f.attributes;
                let ident = f.ident;
                let ty = f.ty;
                attrs.deprecated.map(|target| {
                    deprecated_field_handlers.push(quote! {
                        match (value.#ident.is_some(), value.#target.is_some()) {
                            (true, true) => {
                                Err(mlua::Error::runtime(format!(
                                    "Both '{}' and '{}' are set, use '{}'",
                                    stringify!(#ident),
                                    stringify!(#target),
                                    stringify!(#target)
                                )))
                            }
                            (true, false) => {
                                warn!(
                                    "'{}' field is deprecated, use '{}'",
                                    stringify!(#ident),
                                    stringify!(#target)
                                );
                                std::mem::swap(&mut value.#ident, &mut value.#target);
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

                let default = match (attrs.default, is_option(&ty)) {
                    (Some(default), _) => quote! { Ok(#default) },
                    (None, true) => quote! { Ok(None) },
                    (None, false) => quote! { Err(mlua::Error::runtime("Key not found".to_string())) },
                };

                let initializer = quote! {
                    table.get::<Option<_>>(stringify!(#ident))
                        .and_then(|optional| match optional {
                            Some(x) => #transform,
                            None => #default,
                        })
                        .with_context(|_| {
                            format!("Error occurred when parsing {}.{}", stringify!(#name), stringify!(#ident))
                        })?
                };

                quote! {
                    #ident: #initializer,
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
        Fields::Enum(_enum_field) => unimplemented!(),
    }
}

macro_rules! struct_define {
    ($name:ident {$($body:tt)*} ($variant:ident) $($tail:tt)*) => {
        struct_define!{
            $name
            {
                $($body)*
                $variant: Option<TokenStream>,
            }
            $($tail)*
        }
    };
    ($name:ident {$($body:tt)*} (($variant:ident = $extra:expr)) $($tail:tt)*) => {
        struct_define!{
            $name
            {
                $($body)*
                $variant: Option<TokenStream>,
            }
            $($tail)*
        }
    };
    ($name:ident {$($body:tt)*}) => {
        #[derive(Default)]
        struct $name { $($body)* }
    };
}

macro_rules! struct_init {
    ($this:ident, $attrs:ident, $variant:ident) => {
        $this.$variant = get_attribute(&mut $attrs, stringify!($variant));
    };
    ($this:ident, $attrs:ident, ($variant:ident = $extra:expr)) => {
        $this.$variant = get_attribute_with_default_value(&mut $attrs, stringify!($variant), $extra);
    };
    ($this:ident, $attrs:ident, $($variants:tt),*) => {
        $(struct_init!($this, $attrs, $variants));*
    };
}

macro_rules! define_attrs_struct {
    ($name:ident, $($variants:tt),*) => {
        struct_define!{$name {} $(($variants))*}

        impl $name {
            pub fn parse(attrs: &Vec<Attribute>) -> Self {
                let mut attrs = parse_attributes("mlua", attrs);
                let mut this: Self = Default::default();
                struct_init!{this, attrs, $($variants),*}
                this
            }
        }
    }
}

define_attrs_struct!(
    FieldAttributes,
    (default = quote! {Default::default()}),
    deprecated,
    transform
);

define_attrs_struct!(StructAttributes, validate);

fn get_fields(data: &Data) -> Fields {
    match *data {
        Data::Struct(ref data) => match data.fields {
            syn::Fields::Named(ref fields) => {
                let fields = fields
                    .named
                    .iter()
                    .map(|f| StructField {
                        attributes: FieldAttributes::parse(&f.attrs),
                        ident: f.ident.as_ref().unwrap().clone(),
                        ty: f.ty.clone(),
                    })
                    .collect();
                Fields::Struct(fields)
            }
            syn::Fields::Unnamed(_) | syn::Fields::Unit => unimplemented!(),
        },
        Data::Enum(_) => unimplemented!(),
        Data::Union(_) => unimplemented!(),
    }
}

enum Fields {
    Struct(Vec<StructField>),
    #[allow(dead_code)] // temp
    Enum(EnumField),
}

struct StructField {
    attributes: FieldAttributes,
    ident: Ident,
    ty: Type,
}

struct EnumField {}
