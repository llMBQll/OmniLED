use log::{debug, error};
use mlua::{chunk, Lua, LuaSerdeExt, UserData, Value};
use oled_derive::UniqueUserData;
use serde::Deserialize;
use serde_with::{serde_as, DurationMilliSeconds};
use std::path::PathBuf;
use std::time::Duration;

use crate::common::common::exec_file;
use crate::common::user_data::{UniqueUserData, UserDataRef};
use crate::constants::constants::Constants;
use crate::create_table;
use crate::logging::logger::{LevelFilter, Log};
use crate::renderer::font_selector::FontSelector;

#[serde_as]
#[derive(Deserialize, Debug, UniqueUserData)]
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
        let load_settings_fn = lua
            .create_function(move |lua, settings: Value| {
                let settings: Settings = lua.from_value(settings)?;
                lua.globals().set(Settings::identifier(), settings).unwrap();
                Ok(())
            })
            .unwrap();

        let filename = get_full_path(&Self::settings_file());
        let env = create_table!(lua, {Settings = $load_settings_fn});

        if let Err(err) = exec_file(lua, &filename, env) {
            error!("Error loading settings: {}. Falling back to default", err);

            let default = Value::Table(lua.create_table().unwrap());
            let default: Settings = lua.from_value(default).unwrap();
            lua.globals().set(Settings::identifier(), default).unwrap();
        }

        let settings = UserDataRef::<Settings>::load(lua);
        let logger = UserDataRef::<Log>::load(lua);
        logger.get().set_level_filter(settings.get().log_level);

        debug!("Loaded settings {:?}", settings.get());
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

impl UserData for Settings {}

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
