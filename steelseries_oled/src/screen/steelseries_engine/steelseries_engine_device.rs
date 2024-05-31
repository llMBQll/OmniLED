use crate::renderer::buffer::Buffer;
use mlua::{Lua, Value};

use crate::screen::screen::{MemoryRepresentation, Screen, Settings, Size};
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

    fn update(&mut self, _: &Lua, buffer: Buffer) -> mlua::Result<()> {
        api::update(buffer.bytes());
        Ok(())
    }

    fn name(&mut self, _: &Lua) -> mlua::Result<String> {
        Ok(self.name.clone())
    }

    fn memory_representation(&mut self, _lua: &Lua) -> mlua::Result<MemoryRepresentation> {
        Ok(MemoryRepresentation::BitPerPixel)
    }
}
