use quote::quote;
use syn::DeriveInput;

pub fn expand_lua_name_derive(input: DeriveInput) -> proc_macro::TokenStream {
    let name = input.ident;
    let expanded = quote! {
        impl crate::common::lua_traits::LuaName for #name {
            const NAME: &str = stringify!(#name);
        }
    };
    proc_macro::TokenStream::from(expanded)
}
