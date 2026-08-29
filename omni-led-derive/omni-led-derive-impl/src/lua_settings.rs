use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Fields};

use crate::common::{get_attribute, parse_attributes};

pub fn expand_lua_settings_derive(input: DeriveInput) -> proc_macro::TokenStream {
    let name = input.ident;
    let root = get_struct_attributes(&input.attrs).root;

    let (const_defs, get_set_impls) = expand(input.data, root);

    let expanded = quote! {
        impl #name {
            #const_defs
        }

        impl mlua::FromLua for #name {
            fn from_lua(value: mlua::Value, _lua: &mlua::Lua) -> mlua::Result<Self> {
                match value {
                    mlua::Value::UserData(user_data) => {
                        let value = user_data.borrow::<Self>()?;
                        Ok(value.clone())
                    }
                    other => Err(mlua::Error::FromLuaConversionError {
                        from: other.type_name(),
                        to: String::from(stringify!(#name)),
                        message: None,
                    }),
                }
            }
        }

        impl mlua::UserData for #name {
            fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
                #get_set_impls;
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

fn expand(data: Data, root: TokenStream) -> (TokenStream, TokenStream) {
    let Data::Struct(struct_data) = data else {
        panic!("Expected a struct");
    };
    let Fields::Named(fields) = struct_data.fields else {
        panic!("Expected named fields");
    };

    let fields: Vec<_> = fields
        .named
        .into_iter()
        .map(|f| {
            let attrs = get_field_attributes(&f.attrs);
            let ident = f.ident.unwrap();
            let event_str = format!("{}.{}", root, ident);
            let event_ident = format!("{}_EVENT", ident.to_string().to_ascii_uppercase());
            let event_ident = Ident::new(&event_ident, Span::call_site());

            (attrs, ident, event_ident, event_str)
        })
        .collect();

    let const_defs = fields.iter().map(|(_, _, event_ident, event_str)| {
        quote! {
            const #event_ident: &str = #event_str;
        }
    });
    let const_defs = quote! { #(#const_defs)* };

    let get_set_impls = fields.iter().map(|(attrs, ident, event_ident, _)| {
        let validate = attrs.validate.as_ref().map(|validate| {
            quote! {
                #validate(&val)?;
            }
        });

        quote! {
            fields.add_field_method_get(stringify!(#ident), |_, this| {
                Ok(this.#ident.clone())
            });
            fields.add_field_method_set(stringify!(#ident), |lua, this, val| {
                use mlua::IntoLua;

                #validate
                this.#ident = val;
                crate::events::event_queue::EventQueue::instance()
                    .lock()
                    .unwrap()
                    .push(crate::events::event_queue::Event::Script(crate::events::events::ScriptEvent {
                        event: Self::#event_ident.to_string(),
                        value: this.#ident.clone().into_lua(&lua)?,
                    }));
                Ok(())
            })
        }
    });
    let get_set_impls = quote! { #(#get_set_impls);* };

    (const_defs, get_set_impls)
}

struct StructAttributes {
    root: TokenStream,
}

fn get_struct_attributes(attributes: &Vec<Attribute>) -> StructAttributes {
    let mut attributes = parse_attributes("omni", attributes);

    StructAttributes {
        root: get_attribute(&mut attributes, "root").expect("\"root\" attribute is required"),
    }
}

struct FieldAttributes {
    validate: Option<TokenStream>,
}

fn get_field_attributes(attributes: &Vec<Attribute>) -> FieldAttributes {
    let mut attributes = parse_attributes("omni", attributes);

    FieldAttributes {
        validate: get_attribute(&mut attributes, "validate"),
    }
}
