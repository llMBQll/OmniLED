use mlua::{IntoLua, Lua, OwnedFunction, Value};

pub struct ScopedValue {
    cleanup: OwnedFunction,
}

impl ScopedValue {
    pub fn new<'a, T: IntoLua<'a>>(lua: &'a Lua, name: &str, value: T) -> Self {
        lua.globals().set(name, value).unwrap();

        let name = name.to_string();
        let cleanup = lua
            .create_function(move |lua, _: ()| lua.globals().set(name.clone(), Value::Nil))
            .unwrap()
            .into_owned();

        Self { cleanup }
    }
}

impl Drop for ScopedValue {
    fn drop(&mut self) {
        self.cleanup.call::<_, ()>(()).unwrap()
    }
}
