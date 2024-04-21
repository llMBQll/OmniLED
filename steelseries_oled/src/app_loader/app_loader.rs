use log::{error, warn};
use mlua::{chunk, Lua, UserData};

use crate::app_loader::process::Config;
use crate::common::scoped_value::ScopedValue;
use crate::common::user_data::{UserDataIdentifier, UserDataRef};
use crate::settings::settings::get_full_path;
use crate::{
    app_loader::process::Process, common::common::exec_file, create_table_with_defaults,
    settings::settings::Settings,
};

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

        let env = create_table_with_defaults!(lua, {
            load_app = function(config) APP_LOADER:start_process(config) end,
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

impl UserData for AppLoader {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("start_process", |_lua, this, app_config: Config| {
            this.start_process(app_config);

            Ok(())
        });
    }
}

impl UserDataIdentifier for AppLoader {
    fn identifier() -> &'static str {
        "APP_LOADER"
    }
}
