use mlua::{ChunkMode, Lua, Table};

use crate::common::user_data::UserDataRef;
use crate::constants::constants::Constants;

pub enum ConfigType {
    Applications,
    Devices,
    Scripts,
    Settings,
}

impl ConfigType {
    pub fn get_filename(&self) -> &'static str {
        match self {
            ConfigType::Applications => "applications.lua",
            ConfigType::Devices => "devices.lua",
            ConfigType::Scripts => "scripts.lua",
            ConfigType::Settings => "settings.lua",
        }
    }
}

pub fn read_config(lua: &Lua, config_type: ConfigType) -> mlua::Result<String> {
    let constants = UserDataRef::<Constants>::load(lua);
    let filename = constants.get().config_dir.join(config_type.get_filename());
    std::fs::read_to_string(filename).map_err(mlua::Error::external)
}

pub fn load_config(
    lua: &Lua,
    config_type: ConfigType,
    source: &str,
    env: Table,
) -> mlua::Result<()> {
    lua.load(source)
        .set_mode(ChunkMode::Text)
        .set_name(config_type.get_filename())
        .set_environment(env)
        .exec()
}
