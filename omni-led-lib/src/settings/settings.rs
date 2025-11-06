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

use log::{debug, error};
use mlua::{ErrorContext, FromLua, Lua, UserData, chunk};
use omni_led_derive::{FromLuaValue, UniqueUserData};
use std::time::Duration;

use crate::common::user_data::{UniqueUserData, UserDataRef};
use crate::constants::configs::{ConfigType, Configs};
use crate::create_table_with_defaults;
use crate::logging::logger::{LevelFilter, Log};
use crate::renderer::font_selector::FontSelector;

#[derive(Debug, Clone, UniqueUserData, FromLuaValue)]
pub struct Settings {
    #[mlua(default = 8)]
    pub animation_ticks_delay: usize,

    #[mlua(default = 2)]
    pub animation_ticks_rate: usize,

    #[mlua(default = FontSelector::Default)]
    pub font: FontSelector,

    #[mlua(default = LevelFilter::Info)]
    pub log_level: LevelFilter,

    #[mlua(default = 2)]
    pub keyboard_ticks_repeat_delay: usize,

    #[mlua(default = 2)]
    pub keyboard_ticks_repeat_rate: usize,

    #[mlua(default = 0)]
    pub server_port: u16,

    #[mlua(transform = Self::from_millis)]
    #[mlua(default = Duration::from_millis(100))]
    pub update_interval: Duration,
}

impl Settings {
    pub fn load(lua: &Lua) {
        let load_settings_fn = lua
            .create_function(move |lua, settings: Settings| {
                Settings::set_unique(lua, settings);
                Ok(())
            })
            .unwrap();

        let env = create_table_with_defaults!(lua, {
            LOG = LOG,
            PLATFORM = PLATFORM,
            Settings = $load_settings_fn,
        });

        let result =
            UserDataRef::<Configs>::load(lua)
                .get_mut()
                .load_config(lua, ConfigType::Settings, env);
        if let Err(err) = result {
            error!(
                "Error loading settings: {}. Falling back to default settings",
                err
            );

            let default: Settings = lua.load(chunk! { {} }).eval().unwrap();
            Settings::set_unique(lua, default);
        }

        let settings = UserDataRef::<Settings>::load(lua);
        let logger = UserDataRef::<Log>::load(lua);
        logger.get().set_level_filter(settings.get().log_level);

        debug!("Loaded settings {:?}", settings.get());
    }

    fn from_millis(millis: u64, _: &Lua) -> mlua::Result<Duration> {
        Ok(Duration::from_millis(millis))
    }
}

impl UserData for Settings {}
