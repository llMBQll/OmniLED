use convert_case::Casing;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{Attribute, Data, DeriveInput};

use crate::common::{
    EnumFieldType, collect_enum_variants, get_attribute, get_attribute_with_default_value,
    is_option, parse_attributes,
};

pub fn expand_lua_value_derive(input: DeriveInput) -> proc_macro::TokenStream {
    let name = input.ident;
    let (_impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let struct_attrs = get_struct_attributes(&input.attrs);

    let impl_default = struct_attrs.impl_default.is_some();
    let (initializer, default_initializer, helper_impl) =
        generate_initializer(&name, &input.data, impl_default);

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

    let mut expanded = quote! {
        #helper_impl

        impl mlua::FromLua for #name #ty_generics {
            fn from_lua(value: mlua::Value, lua: &mlua::Lua) -> mlua::Result<#name #ty_generics #where_clause> {
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
                #validate
            }
        }
    };

    if let Some(default_initializer) = default_initializer {
        expanded = quote! {
            #expanded

            impl Default for #name {
                fn default() -> Self {
                    Self {
                        #default_initializer
                        ..Default::default()
                    }
                }
            }
        }
    }

    proc_macro::TokenStream::from(expanded)
}

fn generate_initializer(
    name: &Ident,
    data: &Data,
    impl_default: bool,
) -> (TokenStream, Option<TokenStream>, Option<TokenStream>) {
    match *data {
        Data::Struct(ref data) => match data.fields {
            syn::Fields::Named(ref fields) => {
                let mut init_field: Vec<TokenStream> = Vec::new();
                let mut init_default: Vec<TokenStream> = Vec::new();
                let mut drop_initialized: Vec<TokenStream> = Vec::new();
                let mut impl_defaults: Vec<TokenStream> = Vec::new();
                let mut mask_defs: Vec<TokenStream> = Vec::new();
                let mut mask_refs: Vec<TokenStream> = Vec::new();
                let mut field_names: Vec<TokenStream> = Vec::new();

                for (index, f) in fields.named.iter().enumerate() {
                    let index = index as u64;
                    let field = f.ident.as_ref().unwrap();
                    let attrs = get_field_attributes(&f.attrs);

                    let mask_name = format_ident!(
                        "__FIELD_MASK_{}",
                        field.to_string().to_case(convert_case::Case::UpperSnake)
                    );
                    let mask_def = quote! { const #mask_name: u64 = 1 << #index };
                    let mask_ref = quote! { Self::#mask_name };

                    // for `impl Default for <TYPE>`
                    if let Some(default) = &attrs.default
                        && impl_default
                    {
                        impl_defaults.push(quote! { #field: #default, });
                    }

                    // for `init_field`
                    let write_value = quote! { <_ as mlua::FromLua>::from_lua(value, lua)? };
                    let write_value = match attrs.transform {
                        Some(transform) => {
                            quote! { #transform(#write_value, lua)? }
                        }
                        None => write_value,
                    };

                    init_field.push(quote! {
                        stringify!(#field) => unsafe {
                            (&raw mut (*ptr).#field).write(#write_value);
                            *initialized |= #mask_ref;
                        }
                    });

                    // for `init_defaults`
                    let default = match (attrs.default, is_option(&f.ty)) {
                        (Some(default), _) => Some(default),
                        (None, true) => Some(quote! { None }),
                        (None, false) => None,
                    };

                    if let Some(default) = default {
                        init_default.push(quote! {
                            if *initialized & #mask_ref == 0 {
                                unsafe {
                                    (&raw mut (*ptr).#field).write(#default);
                                    *initialized |= #mask_ref;
                                }
                            }
                        });
                    }

                    // for `drop_initialized`
                    drop_initialized.push(quote! {
                        if initialized & #mask_ref != 0 {
                            unsafe {
                                (&raw mut (*ptr).#field).drop_in_place();
                            }
                        }
                    });

                    // for constant definitions
                    mask_defs.push(mask_def);
                    mask_refs.push(mask_ref);
                    field_names.push(quote! { stringify!(#field) });
                }

                let num_masks = mask_refs.len();
                let mask_all = quote! {
                    const __MASK_ALL: u64 = (1 << #num_masks) - 1
                };
                let mask_map = quote! {
                    const __MASK_MAP: [(u64, &str); #num_masks] = [
                        #( (#mask_refs, #field_names) ),*
                    ]
                };
                let mask_defs = quote! { #(#mask_defs);* };

                let init_field = quote! { #(#init_field)* };
                let init_default = quote! { #(#init_default)* };
                let drop_initialized = quote! { #(#drop_initialized)* };

                let helper_impl = quote! {
                    impl #name {
                        #mask_defs;
                        #mask_all;
                        #mask_map;

                        fn init_field(
                            ptr: *mut Self,
                            initialized: &mut u64,
                            lua: &mlua::Lua,
                            field: &str,
                            value: mlua::Value,
                            unknown: &mut Vec<String>,
                        ) -> mlua::Result<()> {
                            match field {
                                #init_field
                                other => {
                                    unknown.push(other.to_string());
                                }
                            }
                            Ok(())
                        }

                        fn init_default(ptr: *mut Self, initialized: &mut u64) {
                            #init_default
                        }

                        fn drop_initialized(ptr: *mut Self, initialized: *const u64) {
                            let initialized = unsafe { *initialized };
                            #drop_initialized
                        }
                    }
                };

                let initializer = quote! {
                    mlua::Value::Table(table) => {
                        struct DropGuard {
                            ptr: *mut #name,
                            initialized: *const u64,
                        }
                        impl Drop for DropGuard {
                            fn drop(&mut self) {
                                #name::drop_initialized(self.ptr, self.initialized);
                            }
                        }

                        let mut uninit: std::mem::MaybeUninit<Self> = std::mem::MaybeUninit::uninit();
                        let ptr = uninit.as_mut_ptr();
                        let mut initialized = 0;
                        let mut unknown_fields = Vec::new();

                        let drop_guard = DropGuard {
                            ptr,
                            initialized: &mut initialized as *mut _,
                        };

                        for pair in table.pairs() {
                            let (key, value): (String, mlua::Value) = pair?;
                            Self::init_field(ptr, &mut initialized, lua, &key, value, &mut unknown_fields)?;
                        }

                        Self::init_default(ptr, &mut initialized);

                        if !unknown_fields.is_empty() {
                            let unknown_fields = unknown_fields.join(", ");
                            return Err(mlua::Error::runtime(format!(
                                "Unknown fields: [{}]", unknown_fields
                            )));
                        }

                        if initialized != Self::__MASK_ALL {
                            let missing_fields = Self::__MASK_MAP
                                .iter()
                                .filter(|(mask, _)| initialized & mask == 0)
                                .map(|(_, field)| *field)
                                .collect::<Vec<_>>()
                                .join(", ");
                            return Err(mlua::Error::runtime(format!(
                                "Missing fields: [{}]", missing_fields
                            )));
                        }

                        // All errors handled, no need to cleanup the data now
                        std::mem::forget(drop_guard);

                        unsafe { Ok(uninit.assume_init()) }
                    }
                };

                let default_initializers = if impl_default {
                    Some(quote! { #(#impl_defaults)* })
                } else {
                    None
                };

                (initializer, default_initializers, Some(helper_impl))
            }
            syn::Fields::Unnamed(_) | syn::Fields::Unit => unimplemented!(),
        },

        Data::Enum(ref data) => {
            let fields = collect_enum_variants(data, get_enum_attributes);

            let names = fields.iter().map(|(_, ident, _ty, attrs)| {
                let alias = attrs.alias.as_ref().map(|alias| quote! { #alias, });
                quote! {
                    stringify!(#ident), #alias
                }
            });
            let names = quote! { vec![#(#names)*] };

            let mut unnamed_initializers = Vec::new();
            let mut unit_initializers = Vec::new();

            for field in fields {
                match field {
                    (EnumFieldType::Unnamed, ident, _ty, _attrs) => {
                        unnamed_initializers.push(quote! {
                            else if table.contains_key(stringify!(#ident))? {
                                Ok(Self::#ident(table.get(stringify!(#ident))?))
                            }
                        });
                    }
                    (EnumFieldType::Unit, ident, _ty, attrs) => {
                        let alias = attrs.alias.map(|alias| {
                            quote! {
                                #alias => Ok(Self::#ident),
                            }
                        });

                        unit_initializers.push(quote! {
                            stringify!(#ident) => Ok(Self::#ident),
                            #alias
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

            (initializer, None, None)
        }
        Data::Union(_) => unimplemented!(),
    }
}

struct StructAttributes {
    impl_default: Option<TokenStream>,
    validate: Option<TokenStream>,
}

fn get_struct_attributes(attributes: &Vec<Attribute>) -> StructAttributes {
    let mut attributes = parse_attributes("mlua", attributes);

    StructAttributes {
        impl_default: get_attribute_with_default_value(&mut attributes, "impl_default", quote! {}),
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

struct EnumAttributes {
    alias: Option<TokenStream>,
}

fn get_enum_attributes(attributes: &Vec<Attribute>) -> EnumAttributes {
    let mut attributes = parse_attributes("mlua", attributes);

    EnumAttributes {
        alias: get_attribute(&mut attributes, "alias"),
    }
}
