use proc_macro2::{Ident, TokenStream};
use quote::quote;
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
                let mut init_flattened: Vec<TokenStream> = Vec::new();
                let mut drop_initialized: Vec<TokenStream> = Vec::new();
                let mut impl_defaults: Vec<TokenStream> = Vec::new();
                let mut masks: Vec<TokenStream> = Vec::new();
                let mut field_names: Vec<TokenStream> = Vec::new();

                for (index, f) in fields.named.iter().enumerate() {
                    let index = index as u64;
                    let field = f.ident.as_ref().unwrap();
                    let ty = &f.ty;
                    let attrs = get_field_attributes(&f.attrs);

                    let mask = quote! { (1 << #index) };

                    // for `impl Default for <TYPE>`
                    match (&attrs.default, impl_default) {
                        (Some(default), true) => impl_defaults.push(quote! { #field: #default, }),
                        (None, true) => panic!(
                            "Must specify default for '{name}.{field}' when using [mlua(impl_default)]"
                        ),
                        _ => {}
                    };

                    // If flattendedm generate different init code
                    if attrs.flatten.is_some() {
                        let flattened = quote! {
                            unsafe {
                                (&raw mut (*ptr).#field).write(<#ty>::__from_fields(lua, fields, stringify!(#name), false)?);
                                initialized |= #mask;
                            };
                        };
                        init_flattened.push(flattened);
                    } else {
                        // for `init_field`
                        let write_value = quote! { <_ as mlua::FromLua>::from_lua(value, lua) };
                        let write_value = match attrs.transform {
                            Some(transform) => {
                                quote! { #write_value.and_then(|value| #transform(value, lua)) }
                            }
                            None => write_value,
                        };
                        let write_value = quote! {
                            #write_value.with_context(|_| {
                                format!(
                                    "Error occurred when parsing '{}.{}'",
                                    top_level_type, stringify!(#field)
                                )
                            })?
                        };

                        init_field.push(quote! {
                            stringify!(#field) => unsafe {
                                (&raw mut (*ptr).#field).write(#write_value);
                                *initialized |= #mask;
                            }
                        });
                    }

                    // for `init_defaults`
                    let default = match (attrs.default, is_option(ty)) {
                        (Some(default), _) => Some(default),
                        (None, true) => Some(quote! { None }),
                        (None, false) => None,
                    };

                    if let Some(default) = default {
                        init_default.push(quote! {
                            if *initialized & #mask == 0 {
                                unsafe {
                                    (&raw mut (*ptr).#field).write(#default);
                                    *initialized |= #mask;
                                }
                            }
                        });
                    }

                    // for `drop_initialized`
                    drop_initialized.push(quote! {
                        if initialized & #mask != 0 {
                            unsafe {
                                (&raw mut (*ptr).#field).drop_in_place();
                            }
                        }
                    });

                    // for constant definitions
                    masks.push(mask);
                    field_names.push(quote! { stringify!(#field) });
                }

                let num_masks = masks.len();
                let mask_all = quote! {
                    const __MASK_ALL: u64 = (1 << #num_masks) - 1
                };
                let mask_map = quote! {
                    const __MASK_MAP: [(u64, &str); #num_masks] = [
                        #( (#masks, #field_names) ),*
                    ]
                };

                let init_field = quote! { #(#init_field)* };
                let init_default = quote! { #(#init_default)* };
                let init_flattened = quote! { #(#init_flattened)* };
                let drop_initialized = quote! { #(#drop_initialized)* };

                let wrong_fields_error_context = quote! {
                    with_context(|_| {
                        format!("Error occurred when parsing '{}'", top_level_type)
                    })
                };

                let helper_impl = quote! {
                    impl #name {
                        #mask_all;
                        #mask_map;

                        fn __from_fields(
                            lua: &mlua::Lua,
                            fields: &mut Vec<Option<(String, mlua::Value)>>,
                            top_level_type: &'static str,
                            is_top_level: bool,
                        ) -> mlua::Result<Self> {
                            struct DropGuard {
                                ptr: *mut #name,
                                initialized: *const u64,
                            }
                            impl Drop for DropGuard {
                                fn drop(&mut self) {
                                    #name::__drop_initialized(self.ptr, self.initialized);
                                }
                            }

                            let mut uninit: std::mem::MaybeUninit<Self> = std::mem::MaybeUninit::uninit();
                            let ptr = uninit.as_mut_ptr();
                            let mut initialized = 0;

                            let drop_guard = DropGuard {
                                ptr,
                                initialized: &mut initialized as *mut _,
                            };

                            #init_flattened;

                            for field in fields.iter_mut() {
                                Self::__init_field(top_level_type, ptr, &mut initialized, lua, field)?;
                            }

                            Self::__init_default(ptr, &mut initialized);

                            // TODO see if there is a more efficient way to check unknown fields
                            if is_top_level && fields.iter().any(|x| x.is_some()) {
                                use mlua::ErrorContext as _;

                                let unknown_fields = fields
                                    .iter()
                                    .filter_map(|field| field.as_ref().and_then(|(name, _)| Some(name.clone())))
                                    .collect::<Vec<_>>();
                                let unknown_fields = unknown_fields.join(", ");
                                return Err(mlua::Error::runtime(format!(
                                    "Unknown fields: [{}]", unknown_fields
                                )).#wrong_fields_error_context);
                            }

                            if initialized != Self::__MASK_ALL {
                                use mlua::ErrorContext as _;
                                let missing_fields = Self::__MASK_MAP
                                    .iter()
                                    .filter(|(mask, _)| initialized & mask == 0)
                                    .map(|(_, field)| *field)
                                    .collect::<Vec<_>>()
                                    .join(", ");
                                return Err(mlua::Error::runtime(format!(
                                    "Missing fields: [{}]", missing_fields
                                )).#wrong_fields_error_context);
                            }

                            // All errors handled, no need to cleanup the data now
                            std::mem::forget(drop_guard);

                            unsafe { Ok(uninit.assume_init()) }
                        }

                        fn __init_field(
                            top_level_type: &'static str,
                            ptr: *mut Self,
                            initialized: &mut u64,
                            lua: &mlua::Lua,
                            field: &mut Option<(String, mlua::Value)>,
                        ) -> mlua::Result<()> {
                            use mlua::ErrorContext as _;
                            if let Some((name, value)) = field.take() {
                                match name.as_str() {
                                    #init_field
                                    other => {
                                        *field = Some((name, value));
                                    }
                                }
                            }
                            Ok(())
                        }

                        fn __init_default(ptr: *mut Self, initialized: &mut u64) {
                            #init_default
                        }

                        fn __drop_initialized(ptr: *mut Self, initialized: *const u64) {
                            let initialized = unsafe { *initialized };
                            #drop_initialized
                        }
                    }
                };

                let initializer = quote! {
                    mlua::Value::Table(table) => {
                        let mut fields: Vec<Option<(String, mlua::Value)>> = Vec::new();
                        for pair in table.pairs() {
                            let (name, value): (String, mlua::Value) = pair?;
                            fields.push(Some((name, value)));
                        }
                        Self::__from_fields(lua, &mut fields, stringify!(#name), true)
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
    flatten: Option<TokenStream>,
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
        flatten: get_attribute_with_default_value(&mut attributes, "flatten", quote! {}),
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
