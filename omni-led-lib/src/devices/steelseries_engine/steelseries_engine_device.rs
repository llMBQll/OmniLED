use log::error;
use mlua::{ErrorContext, FromLua, Lua, Value};
use omni_led_derive::FromLuaValue;

use crate::devices::device::{Device, MemoryLayout, Settings, Size};
use crate::devices::steelseries_engine::api;
use crate::devices::steelseries_engine::api::Error;
use crate::renderer::buffer::Buffer;

pub struct SteelSeriesEngineDevice {
    name: String,
    size: Size,
}

impl Device for SteelSeriesEngineDevice {
    fn init(lua: &Lua, settings: Value) -> mlua::Result<Self> {
        let settings = SteelSeriesEngineDeviceSettings::from_lua(settings, lua)?;

        let screen_size = settings.screen_size;
        api::register_size(screen_size);

        Ok(Self {
            name: settings.name,
            size: screen_size,
        })
    }

    fn size(&mut self, _: &Lua) -> mlua::Result<Size> {
        Ok(self.size)
    }

    fn update(&mut self, _: &Lua, buffer: Buffer) -> mlua::Result<()> {
        match api::update(&self.size, buffer.bytes()) {
            Ok(_) => Ok(()),
            Err(Error::Disconnected) => {
                error!("SteelSeries Engine is temporarily not available.");
                Ok(())
            }
            Err(Error::NotAvailable(message)) => Err(mlua::Error::runtime(format!(
                "SteelSeries Engine is not available. {}",
                message
            ))),
            Err(Error::BadRequest(error)) => {
                Err(mlua::Error::runtime(format!("Update failed. {:?}", error)))
            }
            Err(Error::BadData(status, response)) => Err(mlua::Error::runtime(format!(
                "Update failed. Response: [{}] {:?}",
                status, response
            ))),
        }
    }

    fn name(&mut self, _: &Lua) -> mlua::Result<String> {
        Ok(self.name.clone())
    }

    fn memory_layout(&mut self, _lua: &Lua) -> mlua::Result<MemoryLayout> {
        Ok(MemoryLayout::BitPerPixel)
    }
}

#[derive(FromLuaValue, Clone)]
pub struct SteelSeriesEngineDeviceSettings {
    pub name: String,
    pub screen_size: Size,
}

impl Settings for SteelSeriesEngineDeviceSettings {
    type DeviceType = SteelSeriesEngineDevice;

    fn name(&self) -> String {
        self.name.clone()
    }
}
