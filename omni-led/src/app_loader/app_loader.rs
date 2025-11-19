use log::{debug, error, warn};
use mlua::{Lua, UserData, chunk};
use omni_led_derive::UniqueUserData;

use crate::app_loader::process::{Config, Process};
use crate::common::common::exec_file;
use crate::common::user_data::{UniqueUserData, UserDataRef};
use crate::constants::constants::Constants;
use crate::create_table_with_defaults;

#[derive(UniqueUserData)]
pub struct AppLoader {
    processes: Vec<Process>,
}

impl AppLoader {
    pub fn load(lua: &Lua) {
        Self::set_unique(
            lua,
            Self {
                processes: Vec::new(),
            },
        );

        let load_app_fn = lua
            .create_function(|lua, config: Config| {
                let mut loader = UserDataRef::<AppLoader>::load(lua);
                loader.get_mut().start_process(config);
                Ok(())
            })
            .unwrap();

        let get_default_path_fn = lua
            .create_function(|lua, app_name: String| {
                let executable = format!("{}{}", app_name, std::env::consts::EXE_SUFFIX);
                let constants = UserDataRef::<Constants>::load(lua);
                let path = constants.get().applications_dir.join(executable);
                Ok(path.to_string_lossy().to_string())
            })
            .unwrap();

        let env = create_table_with_defaults!(lua, {
            load_app = $load_app_fn,
            get_default_path = $get_default_path_fn,
            LOG = LOG,
            PLATFORM = PLATFORM,
            SERVER = SERVER,
        });

        let constants = UserDataRef::<Constants>::load(lua);
        let filename = constants.get().config_dir.join("applications.lua");
        exec_file(lua, &filename, env).unwrap();

        let app_loader = UserDataRef::<AppLoader>::load(lua);
        if app_loader.get().processes.len() == 0 {
            warn!("App loader didn't load any applications");
        }
    }

    fn start_process(&mut self, app_config: Config) {
        match Process::new(&app_config) {
            Ok(process) => {
                debug!("Starting process: {:?}", app_config);
                self.processes.push(process);
            }
            Err(err) => {
                error!("Failed to run {:?}: '{}'", app_config, err);
            }
        }
    }
}

impl UserData for AppLoader {}
