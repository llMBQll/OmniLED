use mlua::{chunk, Lua, LuaSerdeExt, Value};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DurationMilliSeconds};
use std::sync::OnceLock;
use std::time::Duration;

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct NewSettings {
    #[serde(default = "NewSettings::applications_file")]
    pub applications_file: String,

    #[serde(default = "NewSettings::scripts_file")]
    pub scripts_file: String,

    #[serde(default = "NewSettings::scrolling_text_ticks_at_edge")]
    pub scrolling_text_ticks_at_edge: usize,

    #[serde(default = "NewSettings::scrolling_text_ticks_per_move")]
    pub scrolling_text_ticks_per_move: usize,

    #[serde(default = "NewSettings::server_port")]
    pub server_port: u16,

    #[serde(default = "NewSettings::server_port_strict")]
    pub server_port_strict: bool,

    #[serde(default = "NewSettings::settings_file")]
    pub settings_file: String,

    #[serde(default = "NewSettings::supported_screens_file")]
    pub supported_screens_file: String,

    #[serde_as(as = "DurationMilliSeconds")]
    #[serde(default = "NewSettings::update_interval")]
    pub update_interval: Duration,
}

static NEW_SETTINGS: OnceLock<NewSettings> = OnceLock::new();

impl NewSettings {
    pub fn load(lua: &Lua) {
        let filename = Self::settings_file();
        let load_settings = lua
            .create_function(move |lua, settings: Value| {
                let settings: NewSettings = lua.from_value(settings)?;
                Self::set(lua, settings);
                Ok(())
            })
            .unwrap();

        lua.load(chunk! {
            f, err = loadfile($filename, 't', {Settings = $load_settings})
            if err then
                LOG.error(err)
                return
            end
            f()
        })
        .exec()
        .unwrap();
    }

    pub fn get() -> &'static Self {
        NEW_SETTINGS.get().unwrap()
    }

    fn set(lua: &Lua, settings: NewSettings) {
        lua.globals()
            .set("SETTINGS", lua.to_value(&settings).unwrap())
            .unwrap();

        NEW_SETTINGS.get_or_init(move || settings);
    }

    fn applications_file() -> String {
        String::from("applications.lua")
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

impl Default for NewSettings {
    fn default() -> Self {
        Self {
            applications_file: NewSettings::applications_file(),
            scripts_file: NewSettings::scripts_file(),
            scrolling_text_ticks_at_edge: NewSettings::scrolling_text_ticks_at_edge(),
            scrolling_text_ticks_per_move: NewSettings::scrolling_text_ticks_per_move(),
            server_port: NewSettings::server_port(),
            server_port_strict: NewSettings::server_port_strict(),
            settings_file: NewSettings::settings_file(),
            supported_screens_file: NewSettings::supported_screens_file(),
            update_interval: NewSettings::update_interval(),
        }
    }
}
