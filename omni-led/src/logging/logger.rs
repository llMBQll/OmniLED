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
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::{Config, Handle};
use mlua::{FromLua, Lua, UserData, UserDataMethods};
use omni_led_derive::{FromLuaValue, UniqueUserData};
use std::path::{Path, PathBuf};

use crate::common::user_data::UniqueUserData;
use crate::constants::constants::Constants;

#[derive(Clone, Debug, UniqueUserData)]
pub struct Log {
    handle: Handle,
}

impl Log {
    pub fn load(lua: &Lua) {
        let handle = init(Self::get_path());
        let logger = Log { handle };

        Log::set_unique(lua, logger);
    }

    pub fn set_level_filter(&self, level_filter: LevelFilter) {
        change_log_level(&self.handle, Self::get_path(), level_filter.into());
    }

    fn get_path() -> PathBuf {
        Constants::data_dir().join("logging.log")
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

fn init(file_path: impl AsRef<Path>) -> Handle {
    init_with_level(file_path, default_log_level())
}

fn init_with_level(file_path: impl AsRef<Path>, level_filter: log::LevelFilter) -> Handle {
    let config = create_config(file_path, level_filter);
    let handle = log4rs::init_config(config).unwrap();

    std::panic::set_hook(Box::new(|panic_info| {
        error!("{panic_info}");
        println!("{panic_info}");
    }));

    handle
}

fn change_log_level(handle: &Handle, file_path: impl AsRef<Path>, level_filter: log::LevelFilter) {
    let config = create_config(file_path, level_filter);
    handle.set_config(config);
}

fn create_config(file_path: impl AsRef<Path>, level_filter: log::LevelFilter) -> Config {
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "[{d(%Y-%m-%d %H:%M:%S:%3f)}][{l}][{t}] {m}\n",
        )))
        .build(file_path)
        .unwrap();
    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .logger(log4rs::config::Logger::builder().build("mio", log::LevelFilter::Error))
        .logger(log4rs::config::Logger::builder().build("hyper", log::LevelFilter::Error))
        .logger(log4rs::config::Logger::builder().build("rustls", log::LevelFilter::Error))
        .logger(log4rs::config::Logger::builder().build("tracing", log::LevelFilter::Error))
        .logger(log4rs::config::Logger::builder().build("ureq", log::LevelFilter::Error))
        .logger(log4rs::config::Logger::builder().build("warp", log::LevelFilter::Error))
        .build(Root::builder().appender("logfile").build(level_filter))
        .unwrap();

    config
}

#[cfg(debug_assertions)]
fn default_log_level() -> log::LevelFilter {
    log::LevelFilter::Debug
}

#[cfg(not(debug_assertions))]
fn default_log_level() -> log::LevelFilter {
    log::LevelFilter::Info
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
