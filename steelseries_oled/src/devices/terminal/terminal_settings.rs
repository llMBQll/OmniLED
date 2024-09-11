use mlua::{ErrorContext, FromLua, Lua, Value};
use oled_derive::FromLuaValue;

use crate::devices::device::{Settings, Size};

#[derive(FromLuaValue, Clone)]
pub struct TerminalSettings {
    pub name: String,
    pub screen_size: Size,
}

impl Settings for TerminalSettings {
    fn new(lua: &Lua, value: Value) -> mlua::Result<Self> {
        Self::from_lua(value, lua)
    }
}
