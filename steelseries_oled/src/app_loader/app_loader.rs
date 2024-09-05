use log::{error, warn};
use mlua::{chunk, Lua, UserData};
use oled_derive::UniqueUserData;

use crate::app_loader::process::{Config, Process};
use crate::common::common::exec_file;
use crate::common::scoped_value::ScopedValue;
use crate::common::user_data::{UniqueUserData, UserDataRef};
use crate::constants::constants::Constants;
use crate::create_table_with_defaults;
use crate::settings::settings::{get_full_path, Settings};

#[derive(UniqueUserData)]
pub struct AppLoader {
    processes: Vec<Process>,
}

impl AppLoader {
    pub fn load(lua: &Lua) -> ScopedValue {
        let app_loader = ScopedValue::new(
            lua,
            Self::identifier(),
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
            .create_function(|_, app_name: String| {
                let executable = format!("{}{}", app_name, std::env::consts::EXE_SUFFIX);
                let path = Constants::applications_dir().join(executable);
                Ok(path.to_string_lossy().to_string())
            })
            .unwrap();

        let env = create_table_with_defaults!(lua, {
            load_app = $load_app_fn,
            get_default_path = $get_default_path_fn,
            SERVER = SERVER,
            PLATFORM = PLATFORM,
        });

        let settings = UserDataRef::<Settings>::load(lua);
        exec_file(lua, &get_full_path(&settings.get().applications_file), env).unwrap();

        let app_loader_ref = UserDataRef::<AppLoader>::load(lua);
        if app_loader_ref.get().processes.len() == 0 {
            warn!("App loader didn't load any applications");
        }

        app_loader
    }

    fn start_process(&mut self, app_config: Config) {
        match Process::new(&app_config) {
            Ok(process) => {
                self.processes.push(process);
            }
            Err(err) => {
                error!("Failed to run {:?}: '{}'", app_config, err);
            }
        }
    }
}

impl UserData for AppLoader {}
