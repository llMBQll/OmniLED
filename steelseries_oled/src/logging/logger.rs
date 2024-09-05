use log::{debug, error, info, trace, warn};
use log4rs::Handle;
use mlua::{FromLua, Lua, UserData, UserDataMethods};
use oled_derive::{FromLuaValue, UniqueUserData};
use std::path::PathBuf;

use crate::common::user_data::UniqueUserData;
use crate::constants::constants::Constants;

#[derive(Clone, Debug, UniqueUserData)]
pub struct Log {
    handle: Handle,
}

impl Log {
    pub fn load(lua: &Lua) {
        let handle = oled_log::init(Self::get_path());
        let logger = Log { handle };

        lua.globals().set(Log::identifier(), logger).unwrap();
    }

    pub fn set_level_filter(&self, level_filter: LevelFilter) {
        oled_log::change_log_level(&self.handle, Self::get_path(), level_filter.into());
    }

    fn get_path() -> PathBuf {
        Constants::data_dir().join("logging.log")
    }
}

impl UserData for Log {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
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
