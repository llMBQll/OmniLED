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

#[cfg(feature = "unique-user-data")]
mod unique_user_data;

#[cfg(feature = "unique-user-data")]
#[proc_macro_derive(UniqueUserData)]
pub fn unique_user_data_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    unique_user_data::expand_into_unique_user_data_derive(input)
}
