mod common;

#[cfg(feature = "from-lua-value")]
mod from_lua_value;

#[cfg(feature = "from-lua-value")]
#[proc_macro_derive(FromLuaValue, attributes(mlua))]
pub fn from_lua_table_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    from_lua_value::expand_lua_value_derive(input)
}

#[cfg(feature = "lua-enum")]
mod lua_enum;

#[cfg(feature = "lua-enum")]
#[proc_macro_derive(LuaEnum, attributes(mlua))]
pub fn lua_enum_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    lua_enum::expand_lua_enum_derive(input)
}

#[cfg(feature = "into-proto")]
mod into_proto;

#[cfg(feature = "into-proto")]
#[proc_macro_derive(IntoProto, attributes(proto))]
pub fn into_proto_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    into_proto::expand_into_proto_derive(input)
}

#[cfg(feature = "unique-user-data")]
mod unique_user_data;

#[cfg(feature = "unique-user-data")]
#[proc_macro_derive(UniqueUserData)]
pub fn unique_user_data_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    unique_user_data::expand_into_unique_user_data_derive(input)
}
