use std::sync::Arc;
use mlua::{Function, Lua, Nil, Table, TableExt, UserData};
use once_cell::unsync::OnceCell;
use warp::Filter;

pub struct Environment {
    lua: Lua,
}

static EVENTS_STR: &str = "EVENTS";
static EVENT_CLASS_STR: &str = "Event";


struct A<'a> {
    env: &'a Environment,
}

impl <'a> A<'a> {
    pub fn new(env: &'a Environment) -> Self {
        Self {
            env
        }
    }

    pub fn x(&self) {
        self.env.create_event("XD").unwrap();
    }
}

fn xd() {
    let env = Environment::new();
    let a = A::new(&env);
    a.x();
    drop(env);
}

impl Environment {
    pub fn new() -> Self {
        static SCRIPT: &str = include_str!("events.lua");

        let lua = Lua::new();
        lua.load(SCRIPT).exec().unwrap();

        Self {
            lua,
        }
    }

    fn get_events(&self) -> Table {
        self.lua.globals().get(EVENTS_STR).unwrap()
    }

    fn get_event_class(&self) -> Table {
        self.lua.globals().get(EVENT_CLASS_STR).unwrap()
    }

    pub fn create_event(&self, event_name: &str) -> mlua::Result<()> {
        let events = self.get_events();
        let event_class = self.get_event_class();
        let event: Table = event_class.call_method("new", Nil)?;
        events.set(event_name, event)?;

        Ok(())
    }

    pub fn remove_event(&self, event_name: &str) -> mlua::Result<()> {
        let events = self.get_events();
        events.set(event_name, Nil)?;

        Ok(())
    }

    pub fn emit_event<'a, Value: mlua::ToLua<'a>>(&'a self, event_name: &str, value: Value) -> mlua::Result<()> {
        let events = self.get_events();
        let event: Table = events.get(event_name)?;
        event.call_method("emit", value)?;

        Ok(())
    }

    pub fn connect<'a, Slot: mlua::ToLua<'a>>(&'a self, event_name: &str, slot: Slot) -> mlua::Result<()> {
        let events = self.get_events();
        let event: Table = events.get(event_name)?;
        event.call_method("connect", slot)?;

        Ok(())
    }
}