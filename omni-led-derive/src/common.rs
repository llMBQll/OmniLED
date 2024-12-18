/*
 * OmniLED is a software for displaying data on various OLED devices.
 * Copyright (C) 2024  Michał Bałabanow <m.balabanow@gmail.com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use convert_case::Case;
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

pub fn get_case(rename_strategy: &TokenStream) -> Case {
    let strategy = rename_strategy.to_string();
    match strategy.as_str() {
        "lowercase" => Case::Lower,
        "UPPERCASE" => Case::Upper,
        "PascalCase" => Case::Pascal,
        "camelCase" => Case::Camel,
        "snake_case" => Case::Snake,
        "SCREAMING_SNAKE_CASE" => Case::ScreamingSnake,
        "kebab-case" => Case::Kebab,
        "SCREAMING-KEBAB-CASE" => Case::UpperKebab,
        convention => panic!("Unknown case convention '{}'", convention),
    }
}
