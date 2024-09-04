mod common;

#[cfg(feature = "from-lua-value")]
mod from_lua_value;

#[cfg(feature = "from-lua-value")]
#[proc_macro_derive(FromLuaValue, attributes(mlua))]
pub fn from_lua_table_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    from_lua_value::expand_lua_value_derive(input)
}

#[cfg(feature = "into-proto")]
mod into_proto;

#[cfg(feature = "into-proto")]
#[proc_macro_derive(IntoProto, attributes(proto))]
pub fn into_proto_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    into_proto::expand_into_proto_derive(input)
}

#[cfg(feature = "user-data-identifier")]
mod user_data_identifier;

#[cfg(feature = "user-data-identifier")]
#[proc_macro_derive(UserDataIdentifier)]
pub fn user_data_identifier_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    user_data_identifier::expand_into_user_data_identifier_derive(input)
}
