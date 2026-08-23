use log::debug;
use mlua::{Lua, UserData, chunk};
use std::time::Duration;

use crate::common::lua_traits::LuaName;
use crate::common::user_data::{UserDataRef, set_unique_user_data};
use crate::constants::config::{ConfigType, load_config};
use crate::create_table_with_defaults;
use crate::logging::logger::{LevelFilter, Log};
use crate::renderer::font_selector::FontSelector;
use crate::script_handler::script_data_types::DurationWrapper;

#[derive(Debug, Clone, UserData)]
pub struct Settings {
    pub animation_ticks_delay: usize,
    pub animation_ticks_rate: usize,
    pub font: FontSelector,
    pub log_level: LevelFilter,
    pub keyboard_ticks_repeat_delay: usize,
    pub keyboard_ticks_repeat_rate: usize,
    #[lua(skip)]
    pub update_interval: Duration,
}

impl Settings {
    pub fn load(lua: &Lua, config: String) {
        set_unique_user_data(lua, Settings::default());

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

impl Default for Settings {
    fn default() -> Self {
        Self {
            animation_ticks_delay: 8,
            animation_ticks_rate: 2,
            font: FontSelector::Default,
            log_level: LevelFilter::Info,
            keyboard_ticks_repeat_delay: 2,
            keyboard_ticks_repeat_rate: 2,
            update_interval: Duration::from_millis(100),
        }
    }
}

#[mlua::userdata_impl]
impl Settings {
    #[lua(getter, name = "update_interval", infallible)]
    fn get_update_interval(&self) -> DurationWrapper {
        DurationWrapper(self.update_interval)
    }

    #[lua(setter, name = "update_interval", infallible)]
    fn set_update_interval(&mut self, update_interval: DurationWrapper) {
        self.update_interval = update_interval.0;
    }
}

impl LuaName for Settings {
    const NAME: &str = "Settings";
}
