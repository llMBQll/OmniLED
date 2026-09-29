use mlua::{FromLua, Lua, Table, UserData, Value};
use omni_led_derive::LuaName;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::common::lua_traits::LuaName;

#[derive(Clone, LuaName)]
pub struct LuaAccessHandler {
    get_map: Rc<RefCell<HashMap<String, Box<dyn FnMut(&Lua, Table, &str) -> mlua::Result<Value>>>>>,
    set_map:
        Rc<RefCell<HashMap<String, Box<dyn FnMut(&Lua, Table, &str, Value) -> mlua::Result<()>>>>>,
}

impl LuaAccessHandler {
    pub fn instance(lua: &Lua) -> Self {
        let handler: Option<Self> = lua.globals().get(Self::NAME).unwrap();
        match handler {
            Some(handler) => handler,
            None => {
                let handler = Self::new();
                lua.globals().set(Self::NAME, handler.clone()).unwrap();
                handler
            }
        }
    }

    pub fn add_on_get<F: FnMut(&Lua, Table, &str) -> mlua::Result<Value> + 'static>(
        &mut self,
        key: &str,
        handler: F,
    ) {
        self.get_map
            .borrow_mut()
            .insert(key.to_owned(), Box::new(handler));
    }

    pub fn add_on_set<F: FnMut(&Lua, Table, &str, Value) -> mlua::Result<()> + 'static>(
        &mut self,
        key: &str,
        handler: F,
    ) {
        self.set_map
            .borrow_mut()
            .insert(key.to_owned(), Box::new(handler));
    }

    pub fn get(&mut self, lua: &Lua, env: Table, key: &str) -> mlua::Result<Value> {
        match self.get_map.borrow_mut().get_mut(key) {
            Some(handler) => handler(lua, env, key),
            None => env.get(key),
        }
    }

    pub fn set(&mut self, lua: &Lua, env: Table, key: &str, value: Value) -> mlua::Result<()> {
        match self.set_map.borrow_mut().get_mut(key) {
            Some(handler) => handler(lua, env, key, value),
            None => env.set(key, value),
        }
    }

    fn new() -> Self {
        Self {
            get_map: Rc::new(RefCell::new(HashMap::new())),
            set_map: Rc::new(RefCell::new(HashMap::new())),
        }
    }
}

impl UserData for LuaAccessHandler {}

impl FromLua for LuaAccessHandler {
    fn from_lua(value: Value, _lua: &Lua) -> mlua::Result<Self> {
        if let Value::UserData(user_data) = value {
            Ok(user_data.borrow::<Self>()?.clone())
        } else {
            Err(mlua::Error::runtime("Failed to get LuaAccessHandler"))
        }
    }
}
