use std::sync::{Arc, Mutex};
use mlua::{Lua, Nil, Table, TableExt};
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
        let start_application = lua.create_function(move |_, app_config: Config| {
            match Application::new(app_config) {
                Ok(plugin) => {
                    let mut apps = apps.lock().unwrap();
                    apps.push(plugin);
                }
                Err(err) => {
                    println!("{}: '{}'", err, "app_config.path"); // TODO make this accessible
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