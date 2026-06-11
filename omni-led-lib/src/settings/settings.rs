use log::debug;
use mlua::{Lua, UserData, chunk};
use omni_led_derive::FromLuaValue;
use std::time::Duration;

use crate::common::lua_traits::LuaName;
use crate::common::user_data::{UserDataRef, set_unique_user_data};
use crate::constants::config::{ConfigType, load_config};
use crate::create_table_with_defaults;
use crate::logging::logger::{LevelFilter, Log};
use crate::renderer::font_selector::FontSelector;
use crate::script_handler::script_data_types::DurationWrapper;

#[derive(Debug, Clone, FromLuaValue)]
pub struct Settings {
    #[mlua(default = 8)]
    pub animation_ticks_delay: usize,

    #[mlua(default = 2)]
    pub animation_ticks_rate: usize,

    #[mlua(default = FontSelector::Default)]
    pub font: FontSelector,

    #[mlua(default = LevelFilter::Info)]
    pub log_level: LevelFilter,

    #[mlua(default = 2)]
    pub keyboard_ticks_repeat_delay: usize,

    #[mlua(default = 2)]
    pub keyboard_ticks_repeat_rate: usize,

    #[mlua(transform = DurationWrapper::transform)]
    #[mlua(default = Duration::from_millis(100))]
    pub update_interval: Duration,
}

impl Settings {
    pub fn load(lua: &Lua, config: String) {
        let load_settings_fn = lua
            .create_function(move |lua, settings: Settings| {
                set_unique_user_data(lua, settings);
                Ok(())
            })
            .unwrap();

        let env = create_table_with_defaults!(lua, {
            LOG = LOG,
            PLATFORM = PLATFORM,
            Settings = $load_settings_fn,
        });
        load_config(lua, ConfigType::Settings, &config, env).unwrap();

        let settings = UserDataRef::<Settings>::load(lua);
        let logger = UserDataRef::<Log>::load(lua);
        logger.get().set_level_filter(settings.get().log_level);

        debug!("Loaded settings {:?}", settings.get());
    }
}

impl LuaName for Settings {
    const NAME: &str = "SETTINGS";
}

impl UserData for Settings {}
