use mlua::{Lua, Table};

pub struct Logger<'a> {
    lua: &'a Lua,
    log: Table<'a>,
}

impl <'a> Logger<'a> {
    pub fn new(lua: &'a Lua) -> Self {
        static LOG_SRC: &str = include_str!("log.lua");
        lua.load(LOG_SRC).exec().unwrap();

        let log: Table = lua.globals().get("LOG").unwrap();

        Self {
            lua,
            log,
        }
    }
}