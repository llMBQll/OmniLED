use log::{debug, error};
use mlua::{chunk, ErrorContext, FromLua, Lua, UserData};
use oled_derive::{FromLuaValue, UniqueUserData};
use std::path::PathBuf;
use std::time::Duration;

use crate::common::common::exec_file;
use crate::common::user_data::{UniqueUserData, UserDataRef};
use crate::constants::constants::Constants;
use crate::create_table;
use crate::logging::logger::{LevelFilter, Log};
use crate::renderer::font_selector::FontSelector;

#[derive(Debug, Clone, UniqueUserData, FromLuaValue)]
pub struct Settings {
    #[mlua(default(String::from("applications.lua")))]
    pub applications_file: String,

    #[mlua(default(String::from("devices.lua")))]
    pub devices_file: String,

    #[mlua(default(FontSelector::Default))]
    pub font: FontSelector,

    #[mlua(default(LevelFilter::Info))]
    pub log_level: LevelFilter,

    #[mlua(default(2))]
    pub keyboard_ticks_repeat_delay: usize,

    #[mlua(default(2))]
    pub keyboard_ticks_repeat_rate: usize,

    #[mlua(default(String::from("scripts.lua")))]
    pub scripts_file: String,

    #[mlua(default(8))]
    pub scrolling_text_ticks_at_edge: usize,

    #[mlua(default(2))]
    pub scrolling_text_ticks_per_move: usize,

    #[mlua(default(6969))]
    pub server_port: u16,

    #[mlua(transform(Self::from_millis))]
    #[mlua(default(Duration::from_millis(100)))]
    pub update_interval: Duration,
}

impl Settings {
    pub fn load(lua: &Lua) {
        const PATH: &str = "settings.lua";

        let load_settings_fn = lua
            .create_function(move |lua, settings: Settings| {
                lua.globals().set(Settings::identifier(), settings).unwrap();
                Ok(())
            })
            .unwrap();

        let filename = get_full_path(PATH);
        let env = create_table!(lua, {Settings = $load_settings_fn});

        if let Err(err) = exec_file(lua, &filename, env) {
            error!(
                "Error loading settings: {}. Falling back to default settings",
                err
            );

            let default: Settings = lua.load(chunk! { {} }).eval().unwrap();
            lua.globals().set(Settings::identifier(), default).unwrap();
        }

        let settings = UserDataRef::<Settings>::load(lua);
        let logger = UserDataRef::<Log>::load(lua);
        logger.get().set_level_filter(settings.get().log_level);

        debug!("Loaded settings {:?}", settings.get());
    }

    fn from_millis(millis: u64, _: &Lua) -> mlua::Result<Duration> {
        Ok(Duration::from_millis(millis))
    }
}

impl UserData for Settings {}

pub fn get_full_path(path: &str) -> String {
    let path_buf = PathBuf::from(path);
    match path_buf.is_absolute() {
        true => path.to_string(),
        false => Constants::config_dir()
            .join(path)
            .to_str()
            .unwrap()
            .to_string(),
    }
}
