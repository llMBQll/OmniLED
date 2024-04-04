use mlua::{Lua, LuaSerdeExt, Value};
use serde::Deserialize;

use crate::screen::screen::{Error, Result, Settings, Size};

#[derive(Deserialize, Debug)]
pub struct DebugOutputSettings {
    pub name: String,
    pub size: Size,
}

impl Settings for DebugOutputSettings {
    fn new(lua: &Lua, value: Value) -> Result<Self> {
        lua.from_value(value)
            .map_err(|err| Error::InitFailed(err.to_string()))
    }
}
