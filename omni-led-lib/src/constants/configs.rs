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

use mlua::{ErrorContext, Lua, Table, UserData};
use omni_led_derive::UniqueUserData;

use crate::common::user_data::UniqueUserData;

pub trait ConfigProvider {
    fn get_config(&mut self, r#type: ConfigType) -> mlua::Result<String>;
}

#[derive(UniqueUserData)]
pub struct Configs {
    provider: Box<dyn ConfigProvider>,
}

impl Configs {
    pub fn load<P: ConfigProvider + 'static>(lua: &Lua, provider: P) {
        Self::set_unique(
            lua,
            Self {
                provider: Box::new(provider),
            },
        )
    }

    pub fn load_config(&mut self, lua: &Lua, r#type: ConfigType, env: Table) -> mlua::Result<()> {
        let name: &str = r#type.into();
        let cfg = self
            .provider
            .get_config(r#type)
            .with_context(|_| format!("Trying to load {} config", name))?;

        lua.load(cfg)
            .set_name(name)
            .set_environment(env)
            .exec()
            .with_context(|_| format!("Trying to run {} config", name))
    }
}

impl UserData for Configs {}

#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub enum ConfigType {
    Applications,
    Devices,
    Scripts,
    Settings,
}

impl Into<&'static str> for ConfigType {
    fn into(self) -> &'static str {
        match self {
            ConfigType::Applications => "Applications",
            ConfigType::Devices => "Devices",
            ConfigType::Scripts => "Scripts",
            ConfigType::Settings => "Settings",
        }
    }
}
