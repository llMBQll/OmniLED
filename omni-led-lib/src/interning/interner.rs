use mlua::{FromLua, Lua, Value};
use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashSet;
use std::hash::Hash;
use std::ops::Deref;
use std::rc::Rc;

#[derive(Debug)]
pub struct Interned<T: ?Sized>(Rc<T>);

impl<T: ?Sized> Deref for Interned<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T: ?Sized> Clone for Interned<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: ?Sized> PartialEq for Interned<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl<T: ?Sized> Eq for Interned<T> {}

impl<T: ?Sized> Hash for Interned<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let ptr = Rc::as_ptr(&self.0) as *const () as usize;
        ptr.hash(state);
    }
}

impl<T: ?Sized + FromLuaInternEnabled> FromLua for Interned<T> {
    fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
        T::lua_intern(value, lua)
    }
}

pub trait InternEnabled {
    fn intern<I>(value: I) -> Interned<Self>
    where
        I: Sized + Borrow<Self>,
        Rc<Self>: From<I>;
}

macro_rules! enable_interning {
    ($ty:ty) => {
        impl InternEnabled for $ty {
            fn intern<I>(value: I) -> Interned<Self>
            where
                I: Sized + Borrow<Self>,
                Rc<Self>: From<I>,
            {
                thread_local! {
                    static INTERNER: RefCell<Interner<$ty>> =
                        RefCell::new(Interner::new());
                };
                INTERNER.with(|c| c.borrow_mut().intern(value))
            }
        }
    };
}

enable_interning!(str);
enable_interning!([u8]);

trait FromLuaInternEnabled: InternEnabled {
    fn lua_intern(value: Value, lua: &Lua) -> mlua::Result<Interned<Self>>;
}

macro_rules! enable_from_lua_interning {
    ($ty:ty, $intermediate:ty) => {
        impl FromLuaInternEnabled for $ty {
            fn lua_intern(value: Value, lua: &Lua) -> mlua::Result<Interned<Self>> {
                let value = <$intermediate>::from_lua(value, lua)?;
                Ok(Self::intern(value))
            }
        }
    };
}

enable_from_lua_interning!(str, String);
enable_from_lua_interning!([u8], Vec<u8>);

struct Interner<T: ?Sized> {
    values: HashSet<Rc<T>>,
}

impl<T: ?Sized> Interner<T> {
    pub fn new() -> Self {
        Self {
            values: HashSet::new(),
        }
    }
}

impl<T: ?Sized + Hash + Eq> Interner<T> {
    pub fn intern<I>(&mut self, value: I) -> Interned<T>
    where
        I: Sized + Borrow<T>,
        Rc<T>: From<I>,
    {
        if let Some(existing) = self.values.get(value.borrow()) {
            return Interned(existing.clone());
        }
        let value = Rc::from(value);
        self.values.insert(value.clone());
        Interned(value)
    }
}

#[cfg(test)]
mod tests {
    use mlua::chunk;

    use super::*;

    #[test]
    fn intern_value() {
        let mut interner = Interner::<str>::new();

        let a = interner.intern(String::from("Omegalul"));
        let b = interner.intern("Omegalul");

        assert_eq!(interner.values.len(), 1);
        assert_eq!(a.as_ptr(), b.as_ptr());
    }

    #[test]
    fn intern_from_lua() {
        let lua = Lua::new();

        let a: Interned<str> = lua
            .load(chunk! {
                "Omegalul"
            })
            .eval()
            .unwrap();
        let b: Interned<str> = lua
            .load(chunk! {
                "Omegalul"
            })
            .eval()
            .unwrap();

        assert_eq!(a.as_ptr(), b.as_ptr());
    }
}
