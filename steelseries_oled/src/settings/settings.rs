use log::error;
use mlua::{chunk, Lua, LuaSerdeExt, Value};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DurationMilliSeconds};
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::Duration;

use crate::common::common::exec_file;
use crate::common::user_data::UserDataIdentifier;
use crate::constants::constants::Constants;
use crate::create_table;
use crate::renderer::font_selector::FontSelector;

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    #[serde(default = "Settings::applications_file")]
    pub applications_file: String,

    #[serde(default = "Settings::font")]
    pub font: FontSelector,

    #[serde(default = "Settings::scripts_file")]
    pub scripts_file: String,

    #[serde(default = "Settings::scrolling_text_ticks_at_edge")]
    pub scrolling_text_ticks_at_edge: usize,

    #[serde(default = "Settings::scrolling_text_ticks_per_move")]
    pub scrolling_text_ticks_per_move: usize,

    #[serde(default = "Settings::server_port")]
    pub server_port: u16,

    #[serde(default = "Settings::server_port_strict")]
    pub server_port_strict: bool,

    #[serde(default = "Settings::settings_file")]
    pub settings_file: String,

    #[serde(default = "Settings::supported_screens_file")]
    pub supported_screens_file: String,

    #[serde_as(as = "DurationMilliSeconds")]
    #[serde(default = "Settings::update_interval")]
    pub update_interval: Duration,
}

static SETTINGS: OnceLock<Settings> = OnceLock::new();

impl Settings {
    pub fn load(lua: &Lua) {
        let filename = get_full_path(&Self::settings_file());
        let load_settings = lua
            .create_function(move |lua, settings: Value| {
                let settings: Settings = lua.from_value(settings)?;
                Self::set(lua, settings);
                Ok(())
            })
            .unwrap();

        let env = create_table!(lua, {Settings = $load_settings});
        if let Err(err) = exec_file(lua, &filename, env) {
            error!("Couldn't load settings, falling back to default {}", err);
        }
    }

    pub fn get() -> &'static Self {
        SETTINGS.get().unwrap()
    }

    fn set(lua: &Lua, settings: Settings) {
        lua.globals()
            .set(Self::identifier(), lua.to_value(&settings).unwrap())
            .unwrap();

        SETTINGS.get_or_init(move || settings);
    }

    fn applications_file() -> String {
        String::from("applications.lua")
    }

    fn font() -> FontSelector {
        FontSelector::Default
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

    fn server_port_strict() -> bool {
        false
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
            scripts_file: Settings::scripts_file(),
            scrolling_text_ticks_at_edge: Settings::scrolling_text_ticks_at_edge(),
            scrolling_text_ticks_per_move: Settings::scrolling_text_ticks_per_move(),
            server_port: Settings::server_port(),
            server_port_strict: Settings::server_port_strict(),
            settings_file: Settings::settings_file(),
            supported_screens_file: Settings::supported_screens_file(),
            update_interval: Settings::update_interval(),
        }
    }
}

impl UserDataIdentifier for Settings {
    fn identifier() -> &'static str {
        "SETTINGS"
    }
}

pub fn get_full_path(path: &String) -> String {
    let path_buf = PathBuf::from(path);
    match path_buf.is_absolute() {
        true => path.clone(),
        false => Constants::root_dir()
            .join(path)
            .to_str()
            .unwrap()
            .to_string(),
    }
}
