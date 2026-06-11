use mlua::{AnyUserData, IntoLua, Lua, UserData};
use std::marker::PhantomData;

use crate::common::lua_traits::LuaName;

pub fn set_unique_user_data<T: IntoLua + LuaName>(lua: &Lua, value: T) {
    if lua.globals().contains_key(T::NAME).unwrap() {
        panic!("Global value '{}' is already set", T::NAME);
    }

    lua.globals().set(T::NAME, value).unwrap()
}

pub struct UserDataRef<T: UserData + LuaName + 'static> {
    user_data: AnyUserData,
    phantom_data: PhantomData<T>,
}

impl<'a, T: UserData + LuaName + 'static> UserDataRef<T> {
    pub fn load(lua: &Lua) -> Self {
        let user_data = lua.globals().get(T::NAME).unwrap();

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
