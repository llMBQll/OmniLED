use mlua::{ErrorContext, FromLua, Lua, Value};
use oled_derive::FromLuaValue;

use crate::screen::screen::{Settings, Size};

#[derive(FromLuaValue, Clone)]
pub struct DebugOutputSettings {
    pub name: String,
    pub screen_size: Size,
}

impl Settings for DebugOutputSettings {
    fn new(lua: &Lua, value: Value) -> mlua::Result<Self> {
        Self::from_lua(value, lua)
    }
}
