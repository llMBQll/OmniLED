use mlua::{AnyUserData, ErrorContext, FromLua, IntoLua, Lua, Table, UserData, Value};
use std::marker::PhantomData;

use crate::common::lua_access_handler::LuaAccessHandler;
use crate::common::lua_traits::LuaName;

pub fn set_unique_user_data<T: IntoLua + LuaName>(lua: &Lua, value: T) {
    let mut handler = LuaAccessHandler::instance(lua);
    handler.add_on_set(T::NAME, |_: &Lua, _: Table, key: &str, _: Value| {
        Err(mlua::Error::runtime(format!(
            "Global value '{key}' cannot be changed",
        )))
    });

    debug_assert!(
        !lua.globals().contains_key(T::NAME).unwrap(),
        "'{}' already set",
        T::NAME
    );
    lua.globals().set(T::NAME, value).unwrap();
}

pub fn set_mutable_unique_user_data<T: IntoLua + FromLua + LuaName + 'static>(
    lua: &Lua,
    value: T,
    on_set: Option<fn(&Lua, &T) -> mlua::Result<()>>,
) {
    let mut handler = LuaAccessHandler::instance(lua);
    handler.add_on_set(
        T::NAME,
        move |lua: &Lua, env: Table, key: &str, value: Value| {
            let new_value = T::from_lua(value, lua)?;
            if let Some(on_set) = on_set {
                on_set(lua, &new_value)?;
            }
            let user_data: AnyUserData = env.get(key)?;
            let mut original = user_data.borrow_mut::<T>()?;
            *original = new_value;
            Ok(())
        },
    );

    debug_assert!(
        !lua.globals().contains_key(T::NAME).unwrap(),
        "'{}' already set",
        T::NAME
    );
    lua.globals().set(T::NAME, value).unwrap();
}

#[derive(Clone)]
pub struct UserDataRef<T: UserData + LuaName + 'static> {
    user_data: AnyUserData,
    phantom_data: PhantomData<T>,
}

impl<'a, T: UserData + LuaName + 'static> UserDataRef<T> {
    pub fn load(lua: &Lua) -> Self {
        let user_data = lua
            .globals()
            .get(T::NAME)
            .with_context(|_| format!("Failed to find {}", T::NAME))
            .unwrap();

        Self {
            user_data,
            phantom_data: PhantomData,
        }
    }

    pub fn get(&self) -> mlua::UserDataRef<T> {
        self.user_data
            .borrow()
            .with_context(|_| format!("Failed to borrow {}", T::NAME))
            .unwrap()
    }

    pub fn get_mut(&mut self) -> mlua::UserDataRefMut<T> {
        self.user_data
            .borrow_mut()
            .with_context(|_| format!("Failed to mutably borrow {}", T::NAME))
            .unwrap()
    }
}
