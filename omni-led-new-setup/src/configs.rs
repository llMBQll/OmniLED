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

use omni_led_lib::constants::configs::{ConfigProvider, ConfigType};
use std::collections::HashMap;

pub struct ConfigProviderImpl {
    configs: HashMap<ConfigType, String>,
}

impl ConfigProviderImpl {
    pub fn new(configs: HashMap<ConfigType, String>) -> Self {
        Self { configs }
    }
}

impl ConfigProvider for ConfigProviderImpl {
    fn get_config(&mut self, r#type: ConfigType) -> mlua::Result<String> {
        match self.configs.remove(&r#type) {
            Some(config) => Ok(config),
            None => Err(mlua::Error::runtime("Config not found")),
        }
    }
}
