use mlua::{Function, Lua};

pub struct ScriptHandler;

impl ScriptHandler {
    pub fn load(lua: &Lua){
        static SANDBOX_ENV: &str = include_str!("sandbox_env.lua");

        lua.load(SANDBOX_ENV).exec().unwrap();
        lua.globals().get::<_, Function>("compile").unwrap().call::<_, ()>(()).unwrap();
    }
}