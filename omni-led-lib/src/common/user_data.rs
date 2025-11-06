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

use mlua::{AnyUserData, IntoLua, Lua, UserData};
use std::marker::PhantomData;

pub trait UniqueUserData {
    fn identifier() -> &'static str;

    fn set_unique<T: IntoLua + UniqueUserData>(lua: &Lua, value: T) {
        let identifier = T::identifier();

        if lua.globals().contains_key(identifier).unwrap() {
            panic!("Global value '{}' is already set", identifier);
        }

        lua.globals().set(identifier, value).unwrap()
    }
}

pub struct UserDataRef<T: UniqueUserData + UserData + 'static> {
    user_data: AnyUserData,
    phantom_data: PhantomData<T>,
}

impl<'a, T: UniqueUserData + UserData + 'static> UserDataRef<T> {
    pub fn load(lua: &Lua) -> Self {
        let user_data = lua.globals().get(T::identifier()).unwrap();

        Self {
            user_data,
            phantom_data: PhantomData,
        }
    }

    pub fn get(&self) -> mlua::UserDataRef<T> {
        self.user_data.borrow().unwrap()
    }

    pub fn get_mut(&mut self) -> mlua::UserDataRefMut<T> {
        self.user_data.borrow_mut().unwrap()
    }
}
