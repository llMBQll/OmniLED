use mlua::{AnyUserData, Lua, UserData};
use std::cell::{Ref, RefMut};
use std::marker::PhantomData;

pub trait UserDataIdentifier {
    fn identifier() -> &'static str;
}

pub struct UserDataRef<'a, T: UserDataIdentifier + UserData + 'static> {
    user_data: AnyUserData<'a>,
    phantom_data: PhantomData<T>,
}

impl<'a, T: UserDataIdentifier + UserData + 'static> UserDataRef<'a, T> {
    pub fn load(lua: &'a Lua) -> Self {
        let user_data = lua.globals().get(T::identifier()).unwrap();

        Self {
            user_data,
            phantom_data: PhantomData,
        }
    }

    pub fn get(&self) -> Ref<T> {
        self.user_data.borrow::<T>().unwrap()
    }

    pub fn get_mut(&mut self) -> RefMut<T> {
        self.user_data.borrow_mut::<T>().unwrap()
    }
}
