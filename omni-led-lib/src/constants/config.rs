use std::io::Write;

use log::info;
use mlua::{ChunkMode, ErrorContext, Lua, Table};

use crate::constants::constants::Constants;

pub enum ConfigType {
    Devices,
    Plugins,
    Scripts,
    Settings,
}

impl ConfigType {
    fn get_filename(&self) -> &'static str {
        match self {
            ConfigType::Devices => "devices.lua",
            ConfigType::Plugins => "plugins.lua",
            ConfigType::Scripts => "scripts.lua",
            ConfigType::Settings => "settings.lua",
        }
    }

    fn get_default_config(&self) -> &'static str {
        match self {
            ConfigType::Devices => include_str!("../../../config/devices.lua"),
            ConfigType::Plugins => include_str!("../../../config/plugins.lua"),
            ConfigType::Scripts => include_str!("../../../config/scripts.lua"),
            ConfigType::Settings => include_str!("../../../config/settings.lua"),
        }
    }
}

pub fn write_default_configs() -> mlua::Result<()> {
    let config_dir = Constants::config_dir();
    std::fs::create_dir_all(&config_dir)?;

    for config_type in [
        ConfigType::Devices,
        ConfigType::Plugins,
        ConfigType::Scripts,
        ConfigType::Settings,
    ] {
        let path = config_dir.join(config_type.get_filename());
        if let Ok(mut file) = std::fs::File::options()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            file.write_all(config_type.get_default_config().as_bytes())
                .map_err(mlua::Error::external)
                .with_context(|_| format!("Failed to write default config {}", path.display()))?;
            info!("Wrote default config '{}'", path.display());
        }
    }
    Ok(())
}

pub fn read_config(config_type: ConfigType) -> mlua::Result<String> {
    let filename = Constants::config_dir().join(config_type.get_filename());
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
