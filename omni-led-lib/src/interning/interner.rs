use mlua::{FromLua, Lua, Value};
use std::borrow::Borrow;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Deref;
use std::rc::Rc;
use std::time::{Duration, Instant};

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

pub trait InternEnabled: Hash + Eq {
    fn with_interner<R>(f: impl FnOnce(&mut Interner<Self>) -> R) -> R;

    fn intern<I>(value: I) -> Interned<Self>
    where
        I: Sized + Borrow<Self>,
        Rc<Self>: From<I>,
    {
        Self::with_interner(|interner| interner.intern(value))
    }

    fn sweep(threshold: Duration) {
        Self::with_interner(|interner| interner.sweep(threshold));
    }
}

macro_rules! enable_interning {
    ($ty:ty) => {
        impl InternEnabled for $ty {
            fn with_interner<R>(f: impl FnOnce(&mut Interner<$ty>) -> R) -> R {
                thread_local! {
                    static INTERNER: RefCell<Interner<$ty>> =
                        RefCell::new(Interner::new());
                };
                INTERNER.with(|c| f(&mut c.borrow_mut()))
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

pub struct Interner<T: ?Sized> {
    values: HashMap<Rc<T>, Cell<Instant>>,
}

impl<T: ?Sized> Interner<T> {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }
}

impl<T: ?Sized + Hash + Eq> Interner<T> {
    pub fn intern<I>(&mut self, value: I) -> Interned<T>
    where
        I: Sized + Borrow<T>,
        Rc<T>: From<I>,
    {
        if let Some((value, access)) = self.values.get_key_value(value.borrow()) {
            access.set(Instant::now());
            return Interned(value.clone());
        }

        let value = Rc::from(value);
        self.values.insert(value.clone(), Cell::new(Instant::now()));
        Interned(value)
    }

    pub fn sweep(&mut self, threshold: Duration) {
        self.values.retain(|value, access_time| {
            let count = Rc::strong_count(&value);
            // Retain if there are other references **OR** threshlod time hasn't elapsed
            // This is to keep alive interned values inbetween renderer refresh
            count > 1 || access_time.get().elapsed() < threshold
        });
    }
}

#[cfg(test)]
mod tests {
    use mlua::chunk;

    use super::*;

    #[test]
    fn intern_value() {
        let a = <str>::intern(String::from("Omegalul"));
        let b = <str>::intern("Omegalul");

        assert_eq!(<str>::with_interner(|i| i.values.len()), 1);
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

        assert_eq!(<str>::with_interner(|i| i.values.len()), 1);
        assert_eq!(a.as_ptr(), b.as_ptr());
    }

    #[test]
    fn sweep_clears_unused() {
        let a: Interned<str> = <str>::intern("Omegalul");

        <str>::sweep(Duration::ZERO);
        assert_eq!(<str>::with_interner(|i| i.values.len()), 1);

        std::mem::drop(a);
        <str>::sweep(Duration::ZERO);
        assert_eq!(<str>::with_interner(|i| i.values.len()), 0);
    }

    #[test]
    fn sweep_keeps_live_values() {
        let a: Interned<str> = <str>::intern("Omegalul");

        <str>::sweep(Duration::ZERO);
        assert_eq!(<str>::with_interner(|i| i.values.len()), 1);

        let b: Interned<str> = <str>::intern("Omegalul");

        assert_eq!(<str>::with_interner(|i| i.values.len()), 1);
        assert_eq!(a.as_ptr(), b.as_ptr());
    }

    #[test]
    fn sweep_threshold() {
        let a: Interned<str> = <str>::intern("Omegalul");
        std::mem::drop(a);

        <str>::sweep(Duration::from_millis(100));
        assert_eq!(<str>::with_interner(|i| i.values.len()), 1);

        std::thread::sleep(Duration::from_millis(200));
        <str>::sweep(Duration::from_millis(100));
        assert_eq!(<str>::with_interner(|i| i.values.len()), 0);
    }

    #[test]
    fn sweep_touched_entry_threshold() {
        let a: Interned<str> = <str>::intern("Omegalul");
        std::mem::drop(a);

        <str>::sweep(Duration::from_millis(100));
        assert_eq!(<str>::with_interner(|i| i.values.len()), 1);

        std::thread::sleep(Duration::from_millis(200));

        let a: Interned<str> = <str>::intern("Omegalul");
        std::mem::drop(a);

        <str>::sweep(Duration::from_millis(100));
        assert_eq!(<str>::with_interner(|i| i.values.len()), 1);

        std::thread::sleep(Duration::from_millis(200));
        <str>::sweep(Duration::from_millis(100));
        assert_eq!(<str>::with_interner(|i| i.values.len()), 0);
    }
}
