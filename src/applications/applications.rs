use mlua::{Lua, Nil, Table, TableExt};

pub struct Applications<'a> {
    lua: &'a Lua,
    loader: Table<'a>,
}

impl<'a> Applications<'a> {
    pub fn new(lua: &'a Lua) -> Applications<'a> {
        static LOADER_SRC: &str = include_str!("loader.lua");
        lua.load(LOADER_SRC).exec().unwrap();

        let loader: Table = lua.globals().get("LOADER").unwrap();

        Self {
            lua,
            loader,
        }
    }

    pub fn load_applications(&self) -> mlua::Result<()> {
        self.loader.call_function("load_applications", Nil)
    }
}