use std::collections::{HashMap, hash_map::Entry};

use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Attribute, Data, DeriveInput};

use crate::common::{
    EnumFieldType, collect_enum_variants, get_attribute, get_attribute_with_default_value,
    parse_attributes,
};

pub fn expand_lua_enum_derive(input: DeriveInput) -> proc_macro::TokenStream {
    let name = input.ident;
    let table_initializer = generate_initializers(&input.data);
    let (builtin, userdata) = generate_constructors(&input.data);

    let expanded = quote! {
        impl #name {
            pub fn set_lua_enum(lua: &mlua::Lua, env: &mlua::Table) -> mlua::Result<()> {
                let table = lua.create_table()?;
                #table_initializer
                env.set(stringify!(#name), table)?;
                Ok(())
            }
        }

        impl FromLua for #name {
             fn from_lua(value: mlua::Value, _lua: &mlua::Lua) -> mlua::Result<Self> {
                match value {
                    #builtin
                    mlua::Value::UserData(user_data) => {
                        user_data.borrow::<Self>().map(|v| v.clone())
                            #userdata
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

fn generate_initializers(data: &Data) -> TokenStream {
    match *data {
        Data::Struct(_) => panic!("Expected enum"),
        Data::Enum(ref data) => {
            let fields = collect_enum_variants(data, get_enum_attributes);

            let mut initializers = Vec::new();
            let mut constructors = Vec::new();
            for field in fields {
                match field {
                    (EnumFieldType::Unnamed, ident, ty, attrs) => {
                        let initializer = quote! {
                            table.set(stringify!(#ident), lua.create_function(|_, value| {
                                Ok(Self::#ident(value))
                            })?)?;
                        };
                        initializers.push(initializer);

                        if attrs.implicit_construct.is_some() {
                            let constructor = quote! {
                                .or_else(|_| user_data.borrow::<#ty>().map(|x| Self::#ident(x.clone())))
                            };
                            constructors.push(constructor);
                        }
                    }
                    (EnumFieldType::Unit, ident, _ty, attrs) => {
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

fn generate_constructors(data: &Data) -> (TokenStream, TokenStream) {
    match *data {
        Data::Struct(_) => panic!("Expected enum"),
        Data::Enum(ref data) => {
            let mut lua_builtin = HashMap::new();
            let mut userdata = HashMap::new();
            for field in collect_enum_variants(data, get_enum_attributes) {
                match field {
                    (EnumFieldType::Unnamed, ident, ty, attrs) => {
                        if attrs.implicit_construct.is_some() {
                            insert_constructor(&ty, ident, &mut lua_builtin, &mut userdata);
                        }
                    }
                    _ => {}
                }
            }
            let lua_builtin = lua_builtin.into_values();
            let userdata = userdata.into_values();

            (quote! { #(#lua_builtin)* }, quote! { #(#userdata)* })
        }
        Data::Union(_) => panic!("Expected enum"),
    }
}

fn insert_constructor(
    ty: &str,
    ident: &Ident,
    builtin: &mut HashMap<String, TokenStream>,
    userdata: &mut HashMap<String, TokenStream>,
) {
    let (lua_builtin, name, tt) = match ty {
        "bool" => (
            true,
            "bool",
            quote! { mlua::Value::Boolean(value) => Ok(Self::#ident(value)), },
        ),
        integer_ty @ ("i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "usize") => {
            let integer_ty: TokenStream = integer_ty.parse().unwrap();
            (
                true,
                "integer",
                quote! {
                    mlua::Value::Integer(value) =>
                        Ok(Self::#ident(#integer_ty::try_from(value)
                            .map_err(mlua::Error::external)?)),
                },
            )
        }
        number_ty @ ("f32" | "f64") => {
            let number_ty: TokenStream = number_ty.parse().unwrap();
            (
                true,
                "number",
                quote! { mlua::Value::Number(value) => Ok(Self::#ident(value as #number_ty)), },
            )
        }
        "String" => (
            true,
            "String",
            quote! { mlua::Value::String(value) => Ok(Self::#ident(value.to_string_lossy())), },
        ),
        "Table" => (
            true,
            "Table",
            quote! { mlua::Value::Table(value) => Ok(Self::#ident(value)), },
        ),
        "Function" => (
            true,
            "Function",
            quote! { mlua::Value::Function(value) => Ok(Self::#ident(value)), },
        ),
        userdata => {
            let userdata_ty: TokenStream = userdata.parse().unwrap();
            (
                false,
                userdata,
                quote! { .or_else(|_| user_data.borrow::<#userdata_ty>().map(|x| Self::#ident(x.clone()))) },
            )
        }
    };

    let map = if lua_builtin { builtin } else { userdata };
    match map.entry(name.to_string()) {
        Entry::Occupied(_) => {
            panic!("Duplicate type detected: '{name}'");
        }
        Entry::Vacant(vacant_entry) => {
            vacant_entry.insert(tt);
        }
    };
}

struct EnumAttributes {
    alias: Option<TokenStream>,
    implicit_construct: Option<TokenStream>,
}

fn get_enum_attributes(attributes: &Vec<Attribute>) -> EnumAttributes {
    let mut attributes = parse_attributes("mlua", attributes);

    EnumAttributes {
        alias: get_attribute(&mut attributes, "alias"),
        implicit_construct: get_attribute_with_default_value(
            &mut attributes,
            "implicit_construct",
            quote! {},
        ),
    }
}
