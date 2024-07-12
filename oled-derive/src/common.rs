use proc_macro2::TokenStream;
use syn::meta::ParseNestedMeta;
use syn::{parenthesized, token};

pub fn is_attribute(meta: &ParseNestedMeta, attribute: &str) -> bool {
    meta.path.is_ident(attribute)
}

pub fn get_optional_content(meta: &ParseNestedMeta) -> syn::Result<Option<TokenStream>> {
    if meta.input.peek(token::Paren) {
        let content;
        parenthesized!(content in meta.input);
        let s = content.parse()?;
        Ok(Some(s))
    } else {
        Ok(None)
    }
}

pub fn get_content(meta: &ParseNestedMeta) -> syn::Result<TokenStream> {
    match get_optional_content(meta)? {
        Some(content) => Ok(content),
        None => Err(syn::Error::new(
            meta.input.span(),
            format!(
                "'{}' attribute requires an argument",
                meta.path.require_ident()?
            ),
        )),
    }
}
