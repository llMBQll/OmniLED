use log::{debug, error, info, trace, warn};
use mlua::{Lua, UserData, UserDataMethods};
use omni_led_derive::{LuaEnum, LuaName};
use std::collections::HashMap;

use crate::common::lua_traits::{LuaTypeStaticMembers, StaticMembers};
use crate::common::user_data::set_unique_user_data;

pub trait LogImpl {
    fn set_filter_map(&self, filter_map: HashMap<String, LevelFilter>);
}

#[derive(LuaName)]
pub struct Log {
    logger: Box<dyn LogImpl>,
}

impl Log {
    pub fn load<I: LogImpl + 'static>(lua: &Lua, logger: I) {
        set_unique_user_data(
            lua,
            Self {
                logger: Box::new(logger),
            },
        );
    }

    pub fn set_filter_map(&self, filter_map: HashMap<String, LevelFilter>) {
        self.logger.set_filter_map(filter_map);
    }

    fn get_log_location(lua: &Lua) -> String {
        let mut location = String::new();
        let mut level: usize = 1;

        while let Some(Some(name)) = lua.inspect_stack(level, |debug| {
            debug.names().name.map(|name| name.to_string())
        }) {
            location = format!("::{}{}", name, location);
            level += 1;
        }

        let source = match lua.inspect_stack(1, |debug| {
            debug.source().source.map(|source| source.to_string())
        }) {
            Some(Some(source)) => source,
            _ => "script".to_string(),
        };

        format!("{}{}", source, location)
    }
}

impl UserData for Log {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_function("debug", |lua, message: String| {
            debug!(target: &Self::get_log_location(lua), "{}", message);
            Ok(())
        });

        methods.add_function("error", |lua, message: String| {
            error!(target: &Self::get_log_location(lua), "{}", message);
            Ok(())
        });

        methods.add_function("info", |lua, message: String| {
            info!(target: &Self::get_log_location(lua), "{}", message);
            Ok(())
        });

        methods.add_function("trace", |lua, message: String| {
            trace!(target: &Self::get_log_location(lua), "{}", message);
            Ok(())
        });

        methods.add_function("warn", |lua, message: String| {
            warn!(target: &Self::get_log_location(lua), "{}", message);
            Ok(())
        });
    }
}

#[derive(Debug, Copy, Clone, LuaEnum)]
pub enum LevelFilter {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl UserData for LevelFilter {}

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

impl PartialEq<log::Level> for LevelFilter {
    #[inline]
    fn eq(&self, other: &log::Level) -> bool {
        *self as usize == *other as usize
    }
}

impl PartialOrd<log::Level> for LevelFilter {
    #[inline]
    fn partial_cmp(&self, other: &log::Level) -> Option<std::cmp::Ordering> {
        Some((*self as usize).cmp(&(*other as usize)))
    }
}

#[derive(Clone, Debug, LuaName)]
pub struct LogFilterMap {}

impl LogFilterMap {
    pub fn default() -> HashMap<String, LevelFilter> {
        Self::default_filter_map(Self::default_level_filer())
    }

    pub fn default_with(level_filter: LevelFilter) -> HashMap<String, LevelFilter> {
        Self::default_filter_map(level_filter)
    }

    fn default_filter_map(level_filter: LevelFilter) -> HashMap<String, LevelFilter> {
        HashMap::from([
            (String::from("omni_led"), level_filter),
            (String::from("omni_led_api"), level_filter),
            (String::from("omni_led_lib"), level_filter),
            (String::from("devices.lua"), level_filter),
            (String::from("plugins.lua"), level_filter),
            (String::from("scripts.lua"), level_filter),
            (String::from("settings.lua"), level_filter),
            (String::from("script"), level_filter),
            (String::from("plugin"), level_filter),
        ])
    }

    const fn default_level_filer() -> LevelFilter {
        #[cfg(debug_assertions)]
        let level = LevelFilter::Debug;

        #[cfg(not(debug_assertions))]
        let level = LevelFilter::Info;

        level
    }
}

impl LuaTypeStaticMembers for LogFilterMap {
    fn add_members(members: &mut StaticMembers<'_>) {
        members.add_function("default", |_lua, _: ()| Ok(Self::default()));
        members.add_function("default_with", |_lua, level_filter: LevelFilter| {
            Ok(Self::default_with(level_filter))
        });
    }
}

impl UserData for LogFilterMap {}
