use proc_macro2::TokenStream;
use std::collections::HashMap;
use syn::{parenthesized, token, Attribute};

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

                    if meta.input.peek(token::Paren) {
                        let content;
                        parenthesized!(content in meta.input);
                        let stream = content.parse()?;
                        attribute_value = Some(stream);
                    }

                    Ok(())
                })
                .unwrap();

            Some((attribute_name, attribute_value))
        })
        .collect()
}
