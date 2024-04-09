use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{parenthesized, parse_macro_input, token, Attribute, Data, DeriveInput};

#[proc_macro_derive(FromLuaTable, attributes(mlua))]
pub fn from_lua_table_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let (_impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let initializers = generate_initializers(&name, &input.data);

    // TODO handle generics of deriving type, for now only lifetime "'a" is allowed
    let expanded = quote! {
        impl<'a> FromLua<'a> for #name #ty_generics {
            fn from_lua(value: mlua::Value<'a>, lua: &'a mlua::Lua) -> mlua::Result<#name #ty_generics #where_clause> {
                match value {
                    mlua::Value::Table(table) => Ok(#name {
                        #initializers
                    }),
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

fn generate_initializers(name: &Ident, data: &Data) -> TokenStream {
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
                quote! { #(#names)* }
            }
            syn::Fields::Unnamed(_) | syn::Fields::Unit => unimplemented!(),
        },
        Data::Enum(_) | Data::Union(_) => unimplemented!(),
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
            if meta.path.is_ident("default") {
                if meta.input.peek(token::Paren) {
                    let content;
                    parenthesized!(content in meta.input);
                    let s = content.parse()?;
                    default = Some(s);
                } else {
                    default = Some(quote!(Default::default()))
                }
            }
            if meta.path.is_ident("transform") {
                // TODO enforce fn<In: FromLua, Out: T>(in: In, lua: &Lua) -> mlua::Result<Out> { }
                if meta.input.peek(token::Paren) {
                    let content;
                    parenthesized!(content in meta.input);
                    let s = content.parse()?;
                    transform = Some(s);
                } else {
                    panic!("converter expects an argument")
                }
            }

            Ok(())
        })
        .unwrap();
    }

    LuaAttributes { default, transform }
}
