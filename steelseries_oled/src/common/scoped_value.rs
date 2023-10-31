use mlua::{IntoLua, Lua, OwnedFunction};

pub struct ScopedValue {
    cleanup: OwnedFunction,
}

impl ScopedValue {
    pub fn new<'a, T: IntoLua<'a>>(lua: &'a Lua, name: &str, value: T) -> Self {
        lua.globals().set(name, value).unwrap();

        let cleanup = lua
            .load(format!("{} = nil", name))
            .into_function()
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
