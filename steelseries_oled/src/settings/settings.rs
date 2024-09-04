use log::{debug, error, info};
use mlua::{chunk, Lua, LuaSerdeExt, UserData, Value};
use serde::Deserialize;
use serde_with::{serde_as, DurationMilliSeconds};
use std::path::PathBuf;
use std::time::Duration;

use crate::common::common::exec_file;
use crate::common::user_data::{UserDataIdentifier, UserDataRef};
use crate::constants::constants::Constants;
use crate::create_table;
use crate::logging::logger::{LevelFilter, Logger};
use crate::renderer::font_selector::FontSelector;

#[serde_as]
#[derive(Deserialize, Debug)]
pub struct Settings {
    #[serde(default = "Settings::applications_file")]
    pub applications_file: String,

    #[serde(default = "Settings::font")]
    pub font: FontSelector,

    #[serde(default = "Settings::log_level")]
    pub log_level: LevelFilter,

    #[serde(default = "Settings::keyboard_ticks_repeat_delay")]
    pub keyboard_ticks_repeat_delay: usize,

    #[serde(default = "Settings::keyboard_ticks_repeat_rate")]
    pub keyboard_ticks_repeat_rate: usize,

    #[serde(default = "Settings::scripts_file")]
    pub scripts_file: String,

    #[serde(default = "Settings::scrolling_text_ticks_at_edge")]
    pub scrolling_text_ticks_at_edge: usize,

    #[serde(default = "Settings::scrolling_text_ticks_per_move")]
    pub scrolling_text_ticks_per_move: usize,

    #[serde(default = "Settings::server_port")]
    pub server_port: u16,

    #[serde(default = "Settings::supported_screens_file")]
    pub supported_screens_file: String,

    #[serde_as(as = "DurationMilliSeconds")]
    #[serde(default = "Settings::update_interval")]
    pub update_interval: Duration,
}

impl Settings {
    pub fn load(lua: &Lua) {
        let filename = get_full_path(&Self::settings_file());
        let load_settings = lua
            .create_function(move |lua, settings: Value| {
                let settings: Settings = lua.from_value(settings)?;
                lua.globals().set(Settings::identifier(), settings).unwrap();
                Ok(())
            })
            .unwrap();

        let env = create_table!(lua, {Settings = $load_settings});
        if let Err(err) = exec_file(lua, &filename, env) {
            error!("Error loading settings: {}. Falling back to default", err);
            lua.globals()
                .set(Settings::identifier(), Settings::default())
                .unwrap();
        }

        let settings = UserDataRef::<Settings>::load(lua);
        let logger = UserDataRef::<Logger>::load(lua);
        logger.get().set_level_filter(settings.get().log_level);

        debug!("Loaded settings {:?}", settings.get());
        info!(
            "{}",
            serde_json::to_string_pretty(&FontSelector::Default).unwrap()
        );
    }

    fn applications_file() -> String {
        String::from("applications.lua")
    }

    fn font() -> FontSelector {
        FontSelector::Default
    }

    fn log_level() -> LevelFilter {
        LevelFilter(log::LevelFilter::Info)
    }

    fn keyboard_ticks_repeat_delay() -> usize {
        2
    }

    fn keyboard_ticks_repeat_rate() -> usize {
        2
    }

    fn scripts_file() -> String {
        String::from("scripts.lua")
    }

    fn scrolling_text_ticks_at_edge() -> usize {
        8
    }

    fn scrolling_text_ticks_per_move() -> usize {
        2
    }

    fn server_port() -> u16 {
        6969
    }

    fn settings_file() -> String {
        String::from("settings.lua")
    }

    fn supported_screens_file() -> String {
        String::from("screens.lua")
    }

    fn update_interval() -> Duration {
        Duration::from_millis(100)
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            applications_file: Settings::applications_file(),
            font: Settings::font(),
            log_level: Settings::log_level(),
            keyboard_ticks_repeat_delay: Settings::keyboard_ticks_repeat_delay(),
            keyboard_ticks_repeat_rate: Settings::keyboard_ticks_repeat_rate(),
            scripts_file: Settings::scripts_file(),
            scrolling_text_ticks_at_edge: Settings::scrolling_text_ticks_at_edge(),
            scrolling_text_ticks_per_move: Settings::scrolling_text_ticks_per_move(),
            server_port: Settings::server_port(),
            supported_screens_file: Settings::supported_screens_file(),
            update_interval: Settings::update_interval(),
        }
    }
}

impl UserData for Settings {}

impl UserDataIdentifier for Settings {
    fn identifier() -> &'static str {
        "SETTINGS"
    }
}

pub fn get_full_path(path: &String) -> String {
    let path_buf = PathBuf::from(path);
    match path_buf.is_absolute() {
        true => path.clone(),
        false => Constants::config_dir()
            .join(path)
            .to_str()
            .unwrap()
            .to_string(),
    }
}
