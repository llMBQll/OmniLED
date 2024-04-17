use mlua::{ErrorContext, FromLua, Lua, Value};
use oled_derive::FromLuaTable;

use crate::screen::screen::{Settings, Size};

#[derive(FromLuaTable, Clone)]
pub struct DebugOutputSettings {
    pub name: String,
    pub size: Size,
}

impl Settings for DebugOutputSettings {
    fn new(lua: &Lua, value: Value) -> mlua::Result<Self> {
        Self::from_lua(value, lua)
    }
}
