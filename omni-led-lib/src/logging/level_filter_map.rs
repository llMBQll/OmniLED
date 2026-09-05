use mlua::UserData;
use omni_led_derive::LuaName;
use std::collections::HashMap;

use crate::common::lua_traits::{LuaTypeStaticMembers, StaticMembers};
use crate::logging::logger::LevelFilter;

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
