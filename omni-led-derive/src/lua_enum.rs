use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Data, DeriveInput};

use crate::common::{EnumFieldType, collect_enum_variants, get_attribute, parse_attributes};

pub fn expand_lua_enum_derive(input: DeriveInput) -> proc_macro::TokenStream {
    let name = input.ident;
    let initializer = generate_initializer(&input.data);

    let expanded = quote! {
        impl #name {
            pub fn set_lua_enum(lua: &mlua::Lua, env: &mlua::Table) -> mlua::Result<()> {
                let table = lua.create_table()?;
                #initializer
                env.set(stringify!(#name), table)?;
                Ok(())
            }
        }

        impl FromLua for #name {
             fn from_lua(value: mlua::Value, _lua: &mlua::Lua) -> mlua::Result<Self> {
                match value {
                    mlua::Value::UserData(user_data) => {
                        let value = user_data.borrow::<Self>()?;
                        Ok(value.clone())
                    }
                    other => Err(mlua::Error::runtime(format!(
                        "Expected {}, got {}", stringify!(#name), other.type_name()
                    ))),
                }
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

fn generate_initializer(data: &Data) -> TokenStream {
    match *data {
        Data::Struct(_) => panic!("Expected enum"),
        Data::Enum(ref data) => {
            let fields = collect_enum_variants(data, get_enum_attributes);

            let mut initializers = Vec::new();
            for field in fields {
                match field {
                    (EnumFieldType::Unnamed, ident, _attrs) => {
                        let initializer = quote! {
                            table.set(stringify!(#ident), lua.create_function(|_, value| {
                                Ok(Self::#ident(value))
                            })?)?;
                        };
                        initializers.push(initializer);
                    }
                    (EnumFieldType::Unit, ident, attrs) => {
                        let initializer = quote! {
                            table.set(stringify!(#ident), Self::#ident)?;
                        };

                        let alias = attrs.alias.map(|alias| {
                            quote! {
                                table.set(#alias, Self::#ident)?;
                            }
                        });

                        initializers.push(quote! {
                            #initializer
                            #alias
                        });
                    }
                }
            }

            quote! { #(#initializers)* }
        }
        Data::Union(_) => panic!("Expected enum"),
    }
}

struct EnumAttributes {
    alias: Option<TokenStream>,
}

fn get_enum_attributes(attributes: &Vec<Attribute>) -> EnumAttributes {
    let mut attributes = parse_attributes("mlua", attributes);

    EnumAttributes {
        alias: get_attribute(&mut attributes, "alias"),
    }
}
