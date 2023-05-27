use mlua::Lua;

pub struct Events;

impl Events {
    pub fn load(lua: &Lua) {
        static EVENTS_SRC: &str = include_str!("events.lua");
        lua.load(EVENTS_SRC).exec().unwrap();
    }
}