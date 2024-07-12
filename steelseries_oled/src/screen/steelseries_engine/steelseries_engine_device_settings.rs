use mlua::{ErrorContext, FromLua, Lua, Value};
use oled_derive::FromLuaValue;

use crate::screen::screen::{Settings, Size};

#[derive(FromLuaValue, Clone)]
pub struct SteelseriesEngineDeviceSettings {
    pub name: String,
    pub screen_size: Size,
}

impl Settings for SteelseriesEngineDeviceSettings {
    fn new(lua: &Lua, value: Value) -> mlua::Result<Self> {
        Self::from_lua(value, lua)
    }
}
