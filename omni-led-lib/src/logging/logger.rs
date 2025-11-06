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

use log::{debug, error, info, trace, warn};
use mlua::{FromLua, Lua, UserData, UserDataMethods};
use omni_led_derive::{FromLuaValue, UniqueUserData};

use crate::common::user_data::UniqueUserData;

pub trait LogHandle {
    fn set_level_filter(&self, level_filter: log::LevelFilter);
}

#[derive(UniqueUserData)]
pub struct Log {
    handle: Box<dyn LogHandle>,
}

impl Log {
    pub fn load<H: LogHandle + 'static>(lua: &Lua, handle: H) {
        Log::set_unique(
            lua,
            Self {
                handle: Box::new(handle),
            },
        );
    }

    pub fn set_level_filter(&self, level_filter: LevelFilter) {
        self.handle.set_level_filter(level_filter.into());
    }
}

impl UserData for Log {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("debug", |_, _, message: String| {
            debug!("{}", message);
            Ok(())
        });

        methods.add_method("error", |_, _, message: String| {
            error!("{}", message);
            Ok(())
        });

        methods.add_method("info", |_, _, message: String| {
            info!("{}", message);
            Ok(())
        });
        methods.add_method("trace", |_, _, message: String| {
            trace!("{}", message);
            Ok(())
        });

        methods.add_method("warn", |_, _, message: String| {
            warn!("{}", message);
            Ok(())
        });
    }
}

#[derive(Debug, Copy, Clone, FromLuaValue)]
pub enum LevelFilter {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl Into<log::LevelFilter> for LevelFilter {
    fn into(self) -> log::LevelFilter {
        match self {
            LevelFilter::Off => log::LevelFilter::Off,
            LevelFilter::Error => log::LevelFilter::Error,
            LevelFilter::Warn => log::LevelFilter::Warn,
            LevelFilter::Info => log::LevelFilter::Info,
            LevelFilter::Debug => log::LevelFilter::Debug,
            LevelFilter::Trace => log::LevelFilter::Trace,
        }
    }
}
