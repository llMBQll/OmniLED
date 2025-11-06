/*
 * OmniLED is a software for displaying data on various OLED devices.
 * Copyright (C) 2025  Michał Bałabanow <m.balabanow@gmail.com>
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

use mlua::Lua;
use omni_led_lib::common::user_data::UserDataRef;
use omni_led_lib::constants::configs::{ConfigProvider, ConfigType};
use omni_led_lib::constants::constants::Constants;
use std::path::PathBuf;

pub struct ConfigProviderImpl {
    config_dir: PathBuf,
}

impl ConfigProviderImpl {
    pub fn new(lua: &Lua) -> Self {
        let config_dir = UserDataRef::<Constants>::load(lua).get().config_dir.clone();
        Self { config_dir }
    }

    fn get_filename(r#type: ConfigType) -> &'static str {
        match r#type {
            ConfigType::Applications => "applications.lua",
            ConfigType::Devices => "devices.lua",
            ConfigType::Scripts => "scripts.lua",
            ConfigType::Settings => "settings.lua",
        }
    }
}

impl ConfigProvider for ConfigProviderImpl {
    fn get_config(&mut self, r#type: ConfigType) -> mlua::Result<String> {
        let path = self.config_dir.join(Self::get_filename(r#type));
        std::fs::read_to_string(path)
            .map_err(|e| mlua::Error::runtime(format!("{}", e.to_string())))
    }
}
