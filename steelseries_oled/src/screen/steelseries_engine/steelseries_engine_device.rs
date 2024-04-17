use mlua::{Lua, Value};

use crate::screen::screen::{Screen, Settings, Size};
use crate::screen::steelseries_engine::api;
use crate::screen::steelseries_engine::steelseries_engine_device_settings::SteelseriesEngineDeviceSettings;

pub struct SteelseriesEngineDevice {
    name: String,
    size: Size,
}

impl Screen for SteelseriesEngineDevice {
    fn init(lua: &Lua, settings: Value) -> mlua::Result<Self> {
        let settings = SteelseriesEngineDeviceSettings::new(lua, settings)?;

        Ok(Self {
            name: settings.name,
            size: settings.screen_size,
        })
    }

    fn size(&mut self, _: &Lua) -> mlua::Result<Size> {
        Ok(self.size)
    }

    fn update(&mut self, _: &Lua, pixels: Vec<u8>) -> mlua::Result<()> {
        api::update(&pixels);
        Ok(())
    }

    fn name(&mut self, _: &Lua) -> mlua::Result<String> {
        Ok(self.name.clone())
    }
}
