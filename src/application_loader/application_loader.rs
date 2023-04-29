use std::ops::Deref;
use std::sync::{Arc, Mutex};
use mlua::{AnyUserData, Lua, LuaSerdeExt, Nil, Table, TableExt, Value};
use serde::Serialize;
use crate::application_loader::application::{Config, Application};

pub struct ApplicationLoader<'a> {
    lua: &'a Lua,
    loader: Table<'a>,
    applications: Arc<Mutex<Vec<Application>>>,
}

impl<'a> ApplicationLoader<'a> {
    pub fn new(lua: &'a Lua) -> ApplicationLoader<'a> {
        static LOADER_SRC: &str = include_str!("loader.lua");
        lua.load(LOADER_SRC).exec().unwrap();

        let loader: Table = lua.globals().get("LOADER").unwrap();

        let applications = Arc::new(Mutex::new(Vec::new()));
        let mut apps = applications.clone();
        let start_application = lua.create_function(move |lua, app_config: Value| {
            let app_config = lua.from_value(app_config)?;
            match Application::new(&app_config) {
                Ok(plugin) => {
                    let mut apps = apps.lock().unwrap();
                    apps.push(plugin);
                }
                Err(err) => {
                    println!("{}: '{}'", err, serde_json::to_string(&app_config).unwrap());

                }
            }
            Ok(())
        }).unwrap();
        loader.set("start_application", start_application).unwrap();

        Self {
            lua,
            loader,
            applications,
        }
    }

    pub fn load_applications(&self) -> mlua::Result<()> {
        self.loader.call_function("load_applications", Nil)
    }
}