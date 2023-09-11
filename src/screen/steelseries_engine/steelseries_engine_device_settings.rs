use mlua::{Lua, LuaSerdeExt, Value};
use serde::Deserialize;

use crate::screen::screen::{Error, Result, Settings, Size};

#[derive(Deserialize, Debug)]
pub struct SteelseriesEngineDeviceSettings {
    pub name: String,
    pub screen_size: Size,
}

impl Settings for SteelseriesEngineDeviceSettings {
    fn new(lua: &Lua, value: Value) -> Result<Self> {
        lua.from_value(value)
            .map_err(|err| Error::InitFailed(err.to_string()))
    }
}
