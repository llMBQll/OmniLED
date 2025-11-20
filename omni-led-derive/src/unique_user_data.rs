use convert_case::{Case, Casing};
use quote::quote;
use syn::DeriveInput;

pub fn expand_into_unique_user_data_derive(input: DeriveInput) -> proc_macro::TokenStream {
    let name = input.ident;
    let identifier = format!("{}", name);
    let identifier = identifier.to_case(Case::UpperSnake);
    let expanded = quote! {
        impl UniqueUserData for #name {
            fn identifier() -> &'static str {
                #identifier
            }
        }
    };
    proc_macro::TokenStream::from(expanded)
}
