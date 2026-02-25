use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Data, DeriveInput};

use crate::common::{get_attribute, get_attribute_with_default_value, is_option, parse_attributes};

pub fn expand_into_proto_derive(input: DeriveInput) -> proc_macro::TokenStream {
    let name = input.ident;
    let struct_attrs = get_struct_attributes(&input.attrs);
    let assignments = generate_assignments(&input.data, &struct_attrs);

    let expanded = quote! {
        impl Into<omni_led_api::types::Table> for #name {
            fn into(self) -> omni_led_api::types::Table {
                let mut table = omni_led_api::types::Table::default();
                #assignments
                table
            }
        }

        impl Into<omni_led_api::types::Field> for #name {
            fn into(self) -> omni_led_api::types::Field {
                let table = self.into();
                omni_led_api::types::Field {
                    field: Some(omni_led_api::types::field::Field::FTable(table)),
                }
            }
        }
    };
    proc_macro::TokenStream::from(expanded)
}

fn generate_assignments(data: &Data, struct_attrs: &StructAttributes) -> TokenStream {
    match *data {
        Data::Struct(ref data) => match data.fields {
            syn::Fields::Named(ref fields) => {
                let assignments = fields.named.iter().map(|field| {
                    let field_identifier = &field.ident;
                    let field_name = match field_identifier {
                        Some(field) => format!("{}", field),
                        None => String::new(),
                    };

                    let renamed = match &struct_attrs.rename_all {
                        Some(rename_all) => field_name.to_case(get_case(&rename_all)),
                        None => field_name,
                    };

                    let attrs = get_field_attributes(&field.attrs);

                    let is_option = is_option(&field.ty);
                    let propagate_none = attrs.strong_none.is_some();

                    let value_accessor = if is_option {
                        quote! { value }
                    } else {
                        quote! { self.#field_identifier }
                    };

                    let transformed = match attrs.transform {
                        Some(transform) => {
                            quote! { #transform(#value_accessor) }
                        }
                        None => quote! { #value_accessor },
                    };

                    let insertion = quote! {
                        table.items.insert(#renamed.to_string(), #transformed.into())
                    };

                    let none_insertion = quote! {
                        table.items.insert(#renamed.to_string(), omni_led_api::types::None{}.into())
                    };

                    if is_option {
                        if propagate_none {
                            quote! {
                                match self.#field_identifier {
                                    Some(value) => #insertion,
                                    None => #none_insertion,
                                };
                            }
                        } else {
                            quote! {
                                if let Some(value) = self.#field_identifier {
                                    #insertion;
                                }
                            }
                        }
                    } else {
                        quote! { #insertion; }
                    }
                });
                quote! { #(#assignments)* }
            }
            syn::Fields::Unnamed(_) | syn::Fields::Unit => unimplemented!(),
        },
        Data::Enum(_) | Data::Union(_) => unimplemented!(),
    }
}

struct StructAttributes {
    rename_all: Option<TokenStream>,
}

fn get_struct_attributes(attributes: &Vec<Attribute>) -> StructAttributes {
    let mut attributes = parse_attributes("proto", attributes);

    StructAttributes {
        rename_all: get_attribute(&mut attributes, "rename_all"),
    }
}

struct FieldAttributes {
    strong_none: Option<TokenStream>,
    transform: Option<TokenStream>,
}

fn get_field_attributes(attributes: &Vec<Attribute>) -> FieldAttributes {
    let mut attributes = parse_attributes("proto", attributes);

    FieldAttributes {
        strong_none: get_attribute_with_default_value(&mut attributes, "strong_none", quote! {}),
        transform: get_attribute(&mut attributes, "transform"),
    }
}

pub fn get_case(rename_strategy: &TokenStream) -> Case<'_> {
    let strategy = rename_strategy.to_string();
    match strategy.as_str() {
        "lowercase" => Case::Lower,
        "UPPERCASE" => Case::Upper,
        "PascalCase" => Case::Pascal,
        "camelCase" => Case::Camel,
        "snake_case" => Case::Snake,
        "SCREAMING_SNAKE_CASE" => Case::UpperSnake,
        "kebab-case" => Case::Kebab,
        "SCREAMING-KEBAB-CASE" => Case::UpperKebab,
        convention => panic!("Unknown case convention '{}'", convention),
    }
}
