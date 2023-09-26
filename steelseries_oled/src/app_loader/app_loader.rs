use log::{error, warn};
use mlua::{chunk, Lua, LuaSerdeExt, MetaMethod, UserData, Value};

use crate::{
    app_loader::process::Process, common::common::exec_file, create_table,
    settings::settings::Settings,
};

pub struct AppLoader {
    processes: Vec<Process>,
}

impl AppLoader {
    pub fn load(lua: &Lua) {
        let app_loader = Self {
            processes: Vec::new(),
        };
        lua.globals().set("APP_LOADER", app_loader).unwrap();

        let env = create_table!(lua, {
            load_app = function(config) APP_LOADER:start_process(config) end,
            SERVER = SERVER,
            PLATFORM = PLATFORM
        });
        exec_file(lua, &Settings::get().applications_file, env);

        let len: usize = lua.load(chunk! { #APP_LOADER }).eval().unwrap();
        if len == 0 {
            warn!("App loader didn't load any applications")
        }
    }
}

impl UserData for AppLoader {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("start_process", |lua, this, app_config: Value| {
            let app_config = match lua.from_value(app_config) {
                Ok(app_config) => app_config,
                Err(err) => {
                    error!("Failed to parse process config: {}", err);
                    return Ok(());
                }
            };

            match Process::new(&app_config) {
                Ok(process) => {
                    this.processes.push(process);
                }
                Err(err) => {
                    error!(
                        "Failed to run {}: '{}'",
                        serde_json::to_string(&app_config).unwrap(),
                        err
                    );
                }
            }
            Ok(())
        });

        methods.add_meta_method(MetaMethod::Len, |_, this, ()| Ok(this.processes.len()))
    }
}
