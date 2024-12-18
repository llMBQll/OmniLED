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

use mlua::{ErrorContext, FromLua, Lua, Value};
use omni_led_derive::FromLuaValue;

use crate::devices::device::{Settings, Size};

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
