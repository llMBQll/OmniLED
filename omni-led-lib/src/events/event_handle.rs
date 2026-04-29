use mlua::{FromLua, Lua, UserData, Value};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::common::lua_traits::{FromUserdata, LuaName};

#[derive(Clone)]
pub struct EventHandle {
    event_id: Arc<AtomicUsize>,
}

impl EventHandle {
    pub fn new() -> Self {
        Self {
            event_id: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn assign_id(&self, event_id: usize) {
        self.event_id.store(event_id, Ordering::Relaxed);
    }

    pub fn get_id(&self) -> usize {
        self.event_id.load(Ordering::Relaxed)
    }
}

impl PartialEq for EventHandle {
    fn eq(&self, other: &EventHandle) -> bool {
        self.get_id() == other.get_id()
    }
}

impl UserData for EventHandle {}

impl LuaName for EventHandle {
    const NAME: &str = "EventHandle";
}

impl FromUserdata for EventHandle {}

impl FromLua for EventHandle {
    fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
        Self::from_userdata(lua, value)
    }
}
