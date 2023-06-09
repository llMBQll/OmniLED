use std::sync::{Arc, Mutex};
use mlua::{Lua, LuaSerdeExt, Nil, Table, TableExt, Value};

use crate::app_loader::process::Process;

pub struct AppLoader<'a> {
    app_loader: Table<'a>,
    _processes: Arc<Mutex<Vec<Process>>>,
}

impl<'a> AppLoader<'a> {
    pub fn new(lua: &'a Lua) -> AppLoader<'a> {
        static LOADER_SRC: &str = include_str!("app_loader.lua");

        let processes = Arc::new(Mutex::new(Vec::new()));
        let start_process = lua.create_function({
            let processes = Arc::clone(&processes);
            move |lua, app_config: Value| {
                let app_config = lua.from_value(app_config)?;
                match Process::new(&app_config) {
                    Ok(process) => {
                        (*processes.lock().unwrap()).push(process);
                    }
                    Err(err) => {
                        println!("{}: '{}'", err, serde_json::to_string(&app_config).unwrap());
                    }
                }
                Ok(())
            }
        }).unwrap();

        lua.load(LOADER_SRC).exec().unwrap();
        let app_loader: Table = lua.globals().get("APP_LOADER").unwrap();
        app_loader.set("start_process", start_process).unwrap();

        Self {
            app_loader,
            _processes: processes,
        }
    }

    pub fn load(&self) -> mlua::Result<()> {
        self.app_loader.call_function("load_applications", Nil)
    }
}