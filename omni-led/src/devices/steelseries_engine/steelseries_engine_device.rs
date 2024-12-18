/*
 * OmniLED is a software for displaying data on various OLED devices.
 * Copyright (C) 2024  Michał Bałabanow <m.balabanow@gmail.com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use log::error;
use mlua::{Lua, Value};

use crate::devices::device::{Device, MemoryRepresentation, Settings, Size};
use crate::devices::steelseries_engine::api;
use crate::devices::steelseries_engine::api::Error;
use crate::devices::steelseries_engine::steelseries_engine_device_settings::SteelseriesEngineDeviceSettings;
use crate::renderer::buffer::Buffer;

pub struct SteelseriesEngineDevice {
    name: String,
    size: Size,
}

impl Device for SteelseriesEngineDevice {
    fn init(lua: &Lua, settings: Value) -> mlua::Result<Self> {
        let settings = SteelseriesEngineDeviceSettings::new(lua, settings)?;

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
            Err(Error::Disconnected(message)) => {
                error!(
                    "SteelSeries Engine is temporarily not available. {}",
                    message
                );
                Ok(())
            }
            Err(Error::NotAvailable(message)) => Err(mlua::Error::runtime(format!(
                "SteelSeries Engine is not available. {}",
                message
            ))),
            Err(Error::BadRequest(status, response)) => Err(mlua::Error::runtime(format!(
                "Update failed. Response: [{}] {:?}",
                status, response
            ))),
        }
    }

    fn name(&mut self, _: &Lua) -> mlua::Result<String> {
        Ok(self.name.clone())
    }

    fn memory_representation(&mut self, _lua: &Lua) -> mlua::Result<MemoryRepresentation> {
        Ok(MemoryRepresentation::BitPerPixel)
    }
}
