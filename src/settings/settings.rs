use mlua::{FromLua, Lua, Table, TableExt, ToLua};

pub struct Settings;

const SETTINGS: &str = "SETTINGS";

impl Settings {
    pub fn load(lua: &Lua) {
        static SETTINGS_SRC: &str = include_str!("settings.lua");

        lua.load(SETTINGS_SRC).exec().unwrap();
        let settings: Table = lua.globals().get(SETTINGS).unwrap();
        settings.call_method::<_, _, ()>("load", ()).unwrap();
    }

    pub fn get<'a, T: FromLua<'a>>(lua: &'a Lua, key: &str) -> mlua::Result<T> {
        let settings: Table = lua.globals().get(SETTINGS).unwrap();
        settings.get(key)
    }

    #[allow(unused)]
    pub fn set<'a, T: ToLua<'a>>(lua: &'a Lua, key: &str, value: T) -> mlua::Result<()> {
        let settings: Table = lua.globals().get(SETTINGS).unwrap();
        settings.set(key, value)
    }
}