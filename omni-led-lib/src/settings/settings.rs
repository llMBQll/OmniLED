use log::debug;
use mlua::{Lua, UserData};
use omni_led_derive::{DefaultImpl, FromLuaValue};
use std::time::Duration;

use crate::common::lua_traits::LuaName;
use crate::common::sandbox::Sandbox;
use crate::common::user_data::{UserDataRef, set_unique_user_data};
use crate::constants::config::{ConfigType, load_config};
use crate::constants::constants::Constants;
use crate::logging::logger::{LevelFilter, Log};
use crate::renderer::font_selector::FontSelector;
use crate::sandbox_value;
use crate::script_handler::script_data_types::DurationWrapper;
use crate::steelseries_engine::api::ApiSettings;

#[derive(Debug, Clone, DefaultImpl, FromLuaValue)]
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

    #[omni(transform = DurationWrapper::transform)]
    #[omni(default = Duration::from_millis(100))]
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

        let sandbox = Sandbox::new(
            lua,
            vec![
                sandbox_value!(Constants::NAME),
                sandbox_value!(Log::NAME),
                sandbox_value!("Settings", function: load_settings_fn),
            ],
        );
        load_config(lua, ConfigType::Settings, &config, sandbox.proxy).unwrap();

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
