use log::{debug, error, warn};
use mlua::{Lua, UserData, chunk};
use omni_led_derive::UniqueUserData;

use crate::app_loader::c_plugin::{CPlugin, Config};
use crate::common::user_data::{UniqueUserData, UserDataRef};
use crate::constants::config::{ConfigType, load_config};
use crate::constants::constants::Constants;
use crate::create_table_with_defaults;

#[derive(UniqueUserData)]
pub struct AppLoader {
    plugins: Vec<CPlugin>,
}

impl AppLoader {
    pub fn load(lua: &Lua, config: String) {
        Self::set_unique(
            lua,
            Self {
                plugins: Vec::new(),
            },
        );

        let load_plugin_fn = lua
            .create_function(|lua, config: Config| {
                let mut loader = UserDataRef::<AppLoader>::load(lua);
                loader.get_mut().start_plugin(config);
                Ok(())
            })
            .unwrap();

        let get_default_plugin_path_fn = lua
            .create_function(|lua, app_name: String| {
                let executable = format!(
                    "{}{}{}",
                    std::env::consts::DLL_PREFIX,
                    app_name,
                    std::env::consts::DLL_SUFFIX
                );
                let constants = UserDataRef::<Constants>::load(lua);
                let path = constants.get().applications_dir.join(executable);
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

        load_config(lua, ConfigType::Applications, &config, env).unwrap();

        let app_loader = UserDataRef::<AppLoader>::load(lua);
        if app_loader.get().plugins.len() == 0 {
            warn!("App loader didn't load any plugins");
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

impl UserData for AppLoader {}
