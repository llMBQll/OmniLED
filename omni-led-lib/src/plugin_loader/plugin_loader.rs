use log::{debug, error, warn};
use mlua::{Lua, UserData, chunk};
use omni_led_derive::UniqueUserData;

use crate::common::user_data::{UniqueUserData, UserDataRef};
use crate::constants::config::{ConfigType, load_config};
use crate::constants::constants::Constants;
use crate::create_table_with_defaults;
use crate::plugin_loader::c_plugin::{CPlugin, Config};

#[derive(UniqueUserData)]
pub struct PluginLoader {
    plugins: Vec<CPlugin>,
}

impl PluginLoader {
    pub fn load(lua: &Lua, config: String) {
        Self::set_unique(
            lua,
            Self {
                plugins: Vec::new(),
            },
        );

        let load_plugin_fn = lua
            .create_function(|lua, config: Config| {
                let mut loader = UserDataRef::<PluginLoader>::load(lua);
                loader.get_mut().start_plugin(config);
                Ok(())
            })
            .unwrap();

        let get_default_plugin_path_fn = lua
            .create_function(|lua, plugin_name: String| {
                let executable = format!(
                    "{}{}{}",
                    std::env::consts::DLL_PREFIX,
                    plugin_name,
                    std::env::consts::DLL_SUFFIX
                );
                let constants = UserDataRef::<Constants>::load(lua);
                let path = constants.get().plugins_dir.join(executable);
                Ok(path.to_string_lossy().to_string())
            })
            .unwrap();

        let env = create_table_with_defaults!(lua, {
            load_plugin = $load_plugin_fn,
            get_default_plugin_path = $get_default_plugin_path_fn,
            LOG = LOG,
            PLATFORM = PLATFORM,
            SERVER = SERVER,
        });

        load_config(lua, ConfigType::Plugins, &config, env).unwrap();

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
