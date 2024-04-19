use mlua::{FromLua, Lua, UserData, Value};

pub use crate::script_handler::script_data_types::Size;

pub trait Screen {
    fn init(lua: &Lua, settings: Value) -> mlua::Result<Self>
    where
        Self: Sized;

    fn size(&mut self, lua: &Lua) -> mlua::Result<Size>;

    fn update(&mut self, lua: &Lua, pixels: Vec<u8>) -> mlua::Result<()>;

    fn name(&mut self, lua: &Lua) -> mlua::Result<String>;

    // fn partial_update(&mut self, lua: &Lua, pixels: &Vec<u8>) -> Result<()>;
}

pub trait Settings {
    fn new(lua: &Lua, value: Value) -> mlua::Result<Self>
    where
        Self: Sized;
}

impl UserData for Box<dyn Screen> {}

impl<'lua> FromLua<'lua> for Box<dyn Screen> {
    fn from_lua(value: Value<'lua>, _lua: &'lua Lua) -> mlua::Result<Self> {
        match value {
            Value::UserData(user_data) => user_data.take(),
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(),
                to: "Screen",
                message: None,
            }),
        }
    }
}
