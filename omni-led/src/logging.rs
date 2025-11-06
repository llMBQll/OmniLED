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

use log::{LevelFilter, error};
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::{Config, Handle};
use mlua::Lua;
use omni_led_lib::common::user_data::UserDataRef;
use omni_led_lib::constants::constants::Constants;
use omni_led_lib::logging::logger::LogHandle;
use std::path::{Path, PathBuf};

pub struct OmniLedLogHandle {
    handle: Handle,
    path: PathBuf,
}

impl LogHandle for OmniLedLogHandle {
    fn set_level_filter(&self, level_filter: LevelFilter) {
        let config = create_config(&self.path, level_filter);
        self.handle.set_config(config);
    }
}

pub fn init(lua: &Lua) -> OmniLedLogHandle {
    let constants = UserDataRef::<Constants>::load(lua);
    let path = constants.get().data_dir.join("logging.log");

    let config = create_config(&path, default_log_level());
    let handle = log4rs::init_config(config).unwrap();

    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        error!("{panic_info}");
        default_hook(panic_info);
    }));

    OmniLedLogHandle { handle, path }
}

fn create_config(file_path: impl AsRef<Path>, level_filter: LevelFilter) -> Config {
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "[{d(%Y-%m-%d %H:%M:%S:%3f)}][{l}][{t}] {m}\n",
        )))
        .build(file_path)
        .unwrap();
    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .logger(log4rs::config::Logger::builder().build("mio", LevelFilter::Error))
        .logger(log4rs::config::Logger::builder().build("hyper", LevelFilter::Error))
        .logger(log4rs::config::Logger::builder().build("rustls", LevelFilter::Error))
        .logger(log4rs::config::Logger::builder().build("tracing", LevelFilter::Error))
        .logger(log4rs::config::Logger::builder().build("ureq", LevelFilter::Error))
        .logger(log4rs::config::Logger::builder().build("warp", LevelFilter::Error))
        .build(Root::builder().appender("logfile").build(level_filter))
        .unwrap();

    config
}

#[cfg(debug_assertions)]
fn default_log_level() -> LevelFilter {
    LevelFilter::Debug
}

#[cfg(not(debug_assertions))]
fn default_log_level() -> LevelFilter {
    LevelFilter::Info
}
