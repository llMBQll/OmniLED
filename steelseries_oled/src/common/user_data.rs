use mlua::{AnyUserData, IntoLua, Lua, UserData};
use std::marker::PhantomData;

pub trait UniqueUserData {
    fn identifier() -> &'static str;

    fn set_unique<T: IntoLua + UniqueUserData>(lua: &Lua, value: T) {
        if lua.globals().contains_key(T::identifier()).unwrap() {
            panic!("Value '{}' was already set", T::identifier());
        }

        lua.globals().set(T::identifier(), value).unwrap()
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
