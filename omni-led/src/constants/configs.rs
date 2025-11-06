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

use mlua::{ErrorContext, Lua, ObjectLike, Table, UserData, Value};
use omni_led_derive::UniqueUserData;
use std::collections::HashMap;

use crate::common::user_data::{UniqueUserData, UserDataRef};
use crate::constants::constants::Constants;

#[derive(Debug, UniqueUserData)]
pub struct Configs {
    configs: HashMap<&'static str, String>,
}

impl Configs {
    pub fn load(lua: &Lua) {
        Self::set_unique(
            lua,
            Self {
                configs: HashMap::new(),
            },
        )
    }

    pub fn store_config(&mut self, cfg_type: ConfigType, cfg_content: &str) -> mlua::Result<()> {
        let cfg_filename = Self::get_filename(cfg_type);
        self.configs.insert(cfg_filename, cfg_content.to_string());
        Ok(())
    }

    pub fn load_config(&mut self, lua: &Lua, cfg_type: ConfigType, env: Table) -> mlua::Result<()> {
        let cfg_filename = Self::get_filename(cfg_type);

        if let Some(script) = self.configs.remove(cfg_filename) {
            Self::run_memory_script(lua, cfg_filename, script, env)
        } else {
            Self::run_filesystem_script(lua, cfg_filename, env)
        }
    }

    fn run_filesystem_script(lua: &Lua, name: &str, env: Table) -> mlua::Result<()> {
        let config_dir = &UserDataRef::<Constants>::load(&lua).get().config_dir;
        let path = config_dir.join(name);
        let path = path.to_string_lossy().to_string();

        let (function, err): (Value, Value) = lua
            .globals()
            .call_function("loadfile", (path.clone(), "t", env))?;

        let function = match (function, err) {
            (Value::Function(func), Value::Nil) => Ok(func),
            (_, Value::String(err)) => Err(mlua::Error::runtime(err.to_string_lossy())),
            _ => Err(mlua::Error::runtime("Unknown error")),
        };

        function
            .and_then(|function| function.call(()))
            .with_context(|_| format!("Running '{}'", path))
    }

    fn run_memory_script(lua: &Lua, name: &str, script: String, env: Table) -> mlua::Result<()> {
        lua.load(script)
            .set_name(name)
            .set_environment(env)
            .exec()
            .with_context(|_| format!("Running '{}'", name))
    }

    fn get_filename(cfg_type: ConfigType) -> &'static str {
        match cfg_type {
            ConfigType::Applications => "applications.lua",
            ConfigType::Devices => "devices.lua",
            ConfigType::Scripts => "scripts.lua",
            ConfigType::Settings => "settings.lua",
        }
    }
}

impl UserData for Configs {}

pub enum ConfigType {
    Applications,
    Devices,
    Scripts,
    Settings,
}
