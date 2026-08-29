use log::debug;
use mlua::{Lua, chunk};
use omni_led_derive::{DefaultImpl, LuaName, LuaSettings};
use std::time::Duration;

use crate::common::user_data::{UserDataRef, set_unique_user_data};
use crate::constants::config::{ConfigType, load_config};
use crate::create_table_with_defaults;
use crate::logging::logger::{LevelFilter, Log};
use crate::renderer::font_selector::FontSelector;
use crate::script_handler::script_data_types::DurationWrapper;
use crate::steelseries_engine::api::ApiSettings;

#[derive(Debug, Clone, LuaName, LuaSettings, DefaultImpl)]
#[omni(root = Settings)]
pub struct Settings {
    #[omni(default = 8)]
    pub animation_ticks_delay: usize,

    #[omni(default = 2)]
    pub animation_ticks_rate: usize,

    #[omni(default = FontSelector::Default)]
    pub font: FontSelector,

    #[omni(default = LevelFilter::Info)]
    pub log_level: LevelFilter,

    #[omni(default = 2)]
    pub keyboard_ticks_repeat_delay: usize,

    #[omni(default = 2)]
    pub keyboard_ticks_repeat_rate: usize,

    #[omni(default)]
    pub steelseries_api: ApiSettings,

    #[omni(default = DurationWrapper(Duration::from_millis(100)))]
    pub update_interval: DurationWrapper,
}

impl Settings {
    pub fn load(lua: &Lua, config: String) {
        set_unique_user_data(lua, Self::default());

        let env = create_table_with_defaults!(lua, {
            Log = Log,
            PLATFORM = PLATFORM,
            Settings = Settings,
        });
        load_config(lua, ConfigType::Settings, &config, env).unwrap();

        let settings = UserDataRef::<Settings>::load(lua);
        let logger = UserDataRef::<Log>::load(lua);
        logger.get().set_level_filter(settings.get().log_level);

        debug!("Loaded settings {:?}", settings.get());
    }
}
