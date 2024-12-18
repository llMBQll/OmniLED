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

use convert_case::{Case, Casing};
use quote::quote;
use syn::DeriveInput;

pub fn expand_into_unique_user_data_derive(input: DeriveInput) -> proc_macro::TokenStream {
    let name = input.ident;
    // let (_impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    // TODO handle generics of deriving type

    let identifier = format!("{}", name);
    let identifier = identifier.to_case(Case::ScreamingSnake);
    let expanded = quote! {
        impl UniqueUserData for #name {
            fn identifier() -> &'static str {
                #identifier
            }
        }
    };
    proc_macro::TokenStream::from(expanded)
}
