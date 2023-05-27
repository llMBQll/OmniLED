use mlua::{Lua, Table, Value};
use crate::screen::screen::{Error, Screen, ScreenWrapper, Result};
use crate::screen::steelseries_engine::steelseries_engine::SteelseriesEngine;

pub struct Screens;

impl Screens {
    pub fn load(lua: &Lua) {
        let screens = lua.create_table().unwrap();
        lua.globals().set("SCREENS", screens).unwrap();

        Self::register_screen(lua, SteelseriesEngine::new()).unwrap();
    }

    fn register_screen<T: Send + Screen + 'static>(lua: &Lua, mut screen: T) -> Result<()> {
        screen.init()?;

        let name = screen.name()?;
        let screens: Table = lua.globals().get("SCREENS").unwrap();
        match screens.get::<_, Value>(name.clone()).unwrap() {
            Value::Nil => {}
            _ => {
                return Err(Error::NameAlreadyRegistered(name));
            }
        }

        let wrapped = ScreenWrapper::new(screen);
        screens.set(name, wrapped).unwrap();

        Ok(())
    }
}