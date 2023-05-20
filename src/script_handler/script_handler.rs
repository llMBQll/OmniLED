use mlua::{Function, Lua, Table, TableExt};

pub struct ScriptHandler;

impl ScriptHandler {
    pub fn load(lua: &Lua){
        static SANDBOX_ENV: &str = include_str!("script_handler.lua");

        lua.load(SANDBOX_ENV).exec().unwrap();
        let handler: Table = lua.globals().get("SCRIPT_HANDLER").unwrap();
        handler.call_method::<_, _, ()>("compile", ()).unwrap();
    }
}