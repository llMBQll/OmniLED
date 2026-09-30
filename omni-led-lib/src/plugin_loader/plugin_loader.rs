use log::{debug, error, warn};
use mlua::{Lua, UserData};
use omni_led_derive::LuaName;

use crate::common::lua_traits::LuaName;
use crate::common::sandbox::Sandbox;
use crate::common::user_data::{UserDataRef, set_unique_user_data};
use crate::constants::config::{ConfigType, load_config};
use crate::constants::constants::Constants;
use crate::logging::logger::Log;
use crate::plugin_loader::c_plugin::{CPlugin, Config};
use crate::sandbox_value;
use crate::settings::settings::Settings;

#[derive(LuaName)]
pub struct PluginLoader {
    plugins: Vec<CPlugin>,
}

impl PluginLoader {
    pub fn load(lua: &Lua, config: String) {
        set_unique_user_data(
            lua,
            Self {
                plugins: Vec::new(),
            },
        );

        let load_plugin = lua
            .create_function(|lua, config: Config| {
                let mut loader = UserDataRef::<PluginLoader>::load(lua);
                loader.get_mut().start_plugin(config);
                Ok(())
            })
            .unwrap();

        let get_default_plugin_path = lua
            .create_function(|_lua, plugin_name: String| {
                let executable = format!(
                    "{}{}{}",
                    std::env::consts::DLL_PREFIX,
                    plugin_name,
                    std::env::consts::DLL_SUFFIX
                );
                let path = Constants::plugins_dir().join(executable);
                Ok(path.to_string_lossy().to_string())
            })
            .unwrap();

        let sandbox = Sandbox::new(
            lua,
            vec![
                sandbox_value!(Constants::NAME),
                sandbox_value!(Log::NAME),
                sandbox_value!(Settings::NAME),
                sandbox_value!("load_plugin", function: load_plugin),
                sandbox_value!("get_default_plugin_path", function: get_default_plugin_path),
            ],
        );

        load_config(lua, ConfigType::Plugins, &config, sandbox.proxy).unwrap();

        let plugin_loader = UserDataRef::<PluginLoader>::load(lua);
        if plugin_loader.get().plugins.len() == 0 {
            warn!("Plugin loader didn't load any plugins");
        }
    }

    fn start_plugin(&mut self, plugin_config: Config) {
        match CPlugin::new(&plugin_config) {
            Ok(plugin) => {
                debug!("Starting plugin: {:?}", plugin_config);
                self.plugins.push(plugin);
            }
            Err(err) => {
                error!("Failed to run {:?}: '{}'", plugin_config, err);
            }
        }
    }
}

impl UserData for PluginLoader {}
