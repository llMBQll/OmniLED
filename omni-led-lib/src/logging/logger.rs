use log::{debug, error, info, trace, warn};
use mlua::{FromLua, Lua, UserData, UserDataMethods};
use omni_led_derive::{LuaEnum, UniqueUserData};

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
        methods.add_method("debug", |lua, _, message: String| {
            debug!(target: &Self::get_log_location(lua), "{}", message);
            Ok(())
        });

        methods.add_method("error", |lua, _, message: String| {
            error!(target: &Self::get_log_location(lua), "{}", message);
            Ok(())
        });

        methods.add_method("info", |lua, _, message: String| {
            info!(target: &Self::get_log_location(lua), "{}", message);
            Ok(())
        });

        methods.add_method("trace", |lua, _, message: String| {
            trace!(target: &Self::get_log_location(lua), "{}", message);
            Ok(())
        });

        methods.add_method("warn", |lua, _, message: String| {
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
