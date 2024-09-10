use mlua::{Function, IntoLua, Lua, Value};

pub struct ScopedValue {
    cleanup: Function,
}

impl ScopedValue {
    pub fn new<T: IntoLua>(lua: &Lua, name: &str, value: T) -> Self {
        lua.globals().set(name, value).unwrap();

        let name = name.to_string();
        let cleanup = lua
            .create_function(move |lua, _: ()| lua.globals().set(name.clone(), Value::Nil))
            .unwrap();

        Self { cleanup }
    }
}

impl Drop for ScopedValue {
    fn drop(&mut self) {
        self.cleanup.call::<_>(()).unwrap()
    }
}
