use proc_macro2::{Ident, TokenStream};
use std::collections::HashMap;
use syn::{Attribute, DataEnum, Token, Type};

pub fn get_attribute_with_default_value(
    attributes: &mut HashMap<String, Option<TokenStream>>,
    key: &str,
    default: TokenStream,
) -> Option<TokenStream> {
    match attributes.remove(key) {
        Some(content) => Some(content.unwrap_or(default)),
        None => None,
    }
}

pub fn get_attribute(
    attributes: &mut HashMap<String, Option<TokenStream>>,
    key: &str,
) -> Option<TokenStream> {
    match attributes.remove(key) {
        Some(content) => match content {
            Some(content) => Some(content),
            None => panic!("Attribute {key} requires an argument"),
        },
        None => None,
    }
}

pub fn parse_attributes(
    root: &str,
    attributes: &Vec<Attribute>,
) -> HashMap<String, Option<TokenStream>> {
    attributes
        .iter()
        .filter_map(|attribute: &Attribute| {
            if !attribute.path().is_ident(root) {
                return None;
            }

            let mut attribute_name: String = String::new();
            let mut attribute_value: Option<TokenStream> = None;
            attribute
                .parse_nested_meta(|meta| {
                    attribute_name = meta.path.get_ident().unwrap().to_string();

                    if meta.input.peek(Token![=]) {
                        let stream = meta.value()?.parse()?;
                        attribute_value = Some(stream);
                    }

                    Ok(())
                })
                .unwrap();

            Some((attribute_name, attribute_value))
        })
        .collect()
}

pub fn is_option(ty: &Type) -> bool {
    match ty {
        Type::Path(type_path) => type_path.path.segments[0].ident.to_string() == "Option",
        _ => false,
    }
}

#[derive(Debug)]
pub enum EnumFieldType {
    // Named,
    Unnamed,
    Unit,
}

pub fn collect_enum_variants<'a, T: 'static, F: Fn(&Vec<Attribute>) -> T>(
    data: &'a DataEnum,
    get_enum_attributes: F,
) -> Vec<(EnumFieldType, &'a Ident, T)> {
    data.variants
        .iter()
        .map(|variant| match &variant.fields {
            syn::Fields::Named(_) => unimplemented!(),
            syn::Fields::Unnamed(_) => (
                EnumFieldType::Unnamed,
                &variant.ident,
                get_enum_attributes(&variant.attrs),
            ),
            syn::Fields::Unit => (
                EnumFieldType::Unit,
                &variant.ident,
                get_enum_attributes(&variant.attrs),
            ),
        })
        .collect()
}
