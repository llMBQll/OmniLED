use log::debug;
use mlua::Lua;
use omni_led_derive::{DefaultImpl, FromLuaValue, LuaName, LuaSettings};
use std::collections::HashMap;
use std::time::Duration;

use crate::common::lua_traits::LuaName;
use crate::common::sandbox::Sandbox;
use crate::common::user_data::{UserDataRef, set_mutable_unique_user_data};
use crate::constants::config::{ConfigType, load_config};
use crate::constants::constants::Constants;
use crate::logging::level_filter_table::LevelFilterTable;
use crate::logging::logger::{LevelFilter, Log};
use crate::renderer::font_selector::FontSelector;
use crate::sandbox_value;
use crate::script_handler::script_data_types::DurationWrapper;
use crate::steelseries_engine::api::ApiSettings;

#[derive(Debug, Clone, DefaultImpl, FromLuaValue, LuaName, LuaSettings)]
pub struct Settings {
    #[omni(default = 8)]
    pub animation_ticks_delay: usize,

    #[omni(default = 2)]
    pub animation_ticks_rate: usize,

    #[omni(default = FontSelector::Default)]
    pub font: FontSelector,

    #[omni(default = LevelFilterTable::default())]
    #[omni(on_set = Log::set_filter_map)]
    pub level_filter_table: HashMap<String, LevelFilter>,

    #[omni(default = 2)]
    pub keyboard_ticks_repeat_delay: usize,

    #[omni(default = 2)]
    pub keyboard_ticks_repeat_rate: usize,

    #[omni(default)]
    #[omni(on_set = ApiSettings::recursive_set)]
    pub steelseries_api: ApiSettings,

    #[omni(default = DurationWrapper(Duration::from_millis(100)))]
    pub update_interval: DurationWrapper,
}

impl Settings {
    pub fn load(lua: &Lua, config: String) {
        set_mutable_unique_user_data(lua, Self::default(), Some(Self::recursive_set));

        let sandbox = Sandbox::new(
            lua,
            vec![
                sandbox_value!(Constants::NAME),
                sandbox_value!(Log::NAME),
                sandbox_value!(Settings::NAME),
            ],
        );
        load_config(lua, ConfigType::Settings, &config, sandbox.proxy).unwrap();

        let settings = UserDataRef::<Settings>::load(lua);
        debug!("Loaded settings {:?}", settings.get());
    }
}
