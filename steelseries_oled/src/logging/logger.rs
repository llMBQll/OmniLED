use log::{debug, error, info, trace, warn};
use log4rs::Handle;
use mlua::{Lua, UserData, UserDataMethods};
use oled_derive::UserDataIdentifier;
use serde::de;
use serde::de::Error;
use std::path::PathBuf;

use crate::common::user_data::UserDataIdentifier;
use crate::constants::constants::Constants;

#[derive(Clone, Debug, UserDataIdentifier)]
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
        oled_log::change_log_level(&self.handle, Self::get_path(), level_filter.0);
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

#[derive(serde::Deserialize, Debug, Clone, Copy)]
pub struct LevelFilter(#[serde(deserialize_with = "deserialize_log_level")] pub log::LevelFilter);

fn deserialize_log_level<'de, D>(deserializer: D) -> Result<log::LevelFilter, D::Error>
where
    D: de::Deserializer<'de>,
{
    const NAMES: &[&str] = &["Off", "Error", "Warn", "Info", "Debug", "Trace"];

    let s: String = de::Deserialize::deserialize(deserializer)?;
    match s.as_str() {
        "Off" => Ok(log::LevelFilter::Off),
        "Error" => Ok(log::LevelFilter::Error),
        "Warn" => Ok(log::LevelFilter::Warn),
        "Info" => Ok(log::LevelFilter::Info),
        "Debug" => Ok(log::LevelFilter::Debug),
        "Trace" => Ok(log::LevelFilter::Trace),
        value => Err(Error::unknown_variant(value, NAMES)),
    }
}
