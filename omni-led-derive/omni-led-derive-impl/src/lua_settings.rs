use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Fields};

use crate::common::{get_attribute, parse_attributes};

pub fn expand_lua_settings_derive(input: DeriveInput) -> proc_macro::TokenStream {
    let name = input.ident;
    let (get_set_impls, recursive_set) = expand(input.data);

    let expanded = quote! {
        impl #name {
            pub fn recursive_set(lua: &Lua, this: &Self) -> mlua::Result<()> {
                #recursive_set
                Ok(())
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

fn expand(data: Data) -> (TokenStream, TokenStream) {
    let Data::Struct(struct_data) = data else {
        panic!("Expected a struct");
    };
    let Fields::Named(fields) = struct_data.fields else {
        panic!("Expected named fields");
    };

    let mut recursive_set = Vec::new();
    let mut get_set_impls = Vec::new();

    for f in fields.named {
        let attrs = get_field_attributes(&f.attrs);
        let ident = f.ident.unwrap();

        let on_set = attrs.on_set.map(|on_set| {
            quote! {
                #on_set(lua, &this.#ident)?;
            }
        });
        let validate = attrs.validate.map(|validate| {
            quote! {
                #validate(lua, &this.#ident)?;
            }
        });

        recursive_set.push(quote! {
            #validate
            #on_set
        });

        get_set_impls.push(quote! {
            fields.add_field_method_get(stringify!(#ident), |_, this| {
                Ok(this.#ident.clone())
            });
            fields.add_field_method_set(stringify!(#ident), |lua, this, val| {
                this.#ident = val;
                #validate
                #on_set
                Ok(())
            })
        });
    }

    (
        quote! { #(#get_set_impls);* },
        quote! { #(#recursive_set)* },
    )
}

struct FieldAttributes {
    on_set: Option<TokenStream>,
    validate: Option<TokenStream>,
}

fn get_field_attributes(attributes: &Vec<Attribute>) -> FieldAttributes {
    let mut attributes = parse_attributes("omni", attributes);

    FieldAttributes {
        on_set: get_attribute(&mut attributes, "on_set"),
        validate: get_attribute(&mut attributes, "validate"),
    }
}
