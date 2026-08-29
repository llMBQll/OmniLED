mod common;

#[cfg(feature = "default-impl")]
mod default_impl;

#[cfg(feature = "default-impl")]
#[proc_macro_derive(DefaultImpl, attributes(omni))]
pub fn default_impl_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    default_impl::expand_default_impl_derive(input)
}

#[cfg(feature = "from-lua-value")]
mod from_lua_value;

#[cfg(feature = "from-lua-value")]
#[proc_macro_derive(FromLuaValue, attributes(omni))]
pub fn from_lua_table_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    from_lua_value::expand_lua_value_derive(input)
}

#[cfg(feature = "lua-enum")]
mod lua_enum;

#[cfg(feature = "lua-enum")]
#[proc_macro_derive(LuaEnum, attributes(omni))]
pub fn lua_enum_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    lua_enum::expand_lua_enum_derive(input)
}

#[cfg(feature = "lua-name")]
mod lua_name;

#[cfg(feature = "lua-name")]
#[proc_macro_derive(LuaName, attributes(omni))]
pub fn lua_name_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    lua_name::expand_lua_name_derive(input)
}

#[cfg(feature = "lua-settings")]
mod lua_settings;

#[cfg(feature = "lua-settings")]
#[proc_macro_derive(LuaSettings, attributes(omni))]
pub fn lua_settings_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    lua_settings::expand_lua_settings_derive(input)
}

#[cfg(feature = "plugin-entry")]
mod plugin_entry;

#[cfg(feature = "plugin-entry")]
#[proc_macro_attribute]
pub fn plugin_entry(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    plugin_entry::expand_plugin_entry_attr(attr.into(), item.into()).into()
}
