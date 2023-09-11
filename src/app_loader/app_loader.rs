use log::error;
use mlua::{chunk, Lua, LuaSerdeExt, MetaMethod, UserData, Value};

use crate::app_loader::process::Process;

pub struct AppLoader {
    processes: Vec<Process>,
}

impl AppLoader {
    pub fn load(lua: &Lua) {
        let app_loader = Self {
            processes: Vec::new(),
        };
        lua.load(chunk! {
            APP_LOADER = $app_loader

            f, err = loadfile(SETTINGS.applications_file, 't', {
                load_app = function(config) APP_LOADER:start_process(config) end,
                SERVER = SERVER,
                PLATFORM = PLATFORM
            })
            if err then
                // this is not a fatal error since there can also be
                // external applications that will not ne loaded here
                LOG.error(err)
            else
                f()
            end

            if #APP_LOADER == 0 then
                LOG.warn("App loader didn't load any applications")
            end
        })
        .exec()
        .unwrap();
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
