use mlua::{Function, Lua, Table, Value};
use std::collections::{HashMap, HashSet};
use std::time::Duration;

use crate::model::display::Display;
use crate::model::operation::{Bar, Operation, ScrollingText, Text, Modifiers};
use crate::model::position::Position;

pub struct ScriptHandler {
    lua: Lua,
    sensitivity_lists: Vec<(String, HashSet<String>)>,
}

impl ScriptHandler {
    pub fn new() -> Self {
        let lua = Lua::new();
        Self::load_sandbox(&lua);

        Self {
            lua,
            sensitivity_lists: Vec::new(),
        }
    }

    pub fn register_display(&mut self, displays: Vec<Display>) -> mlua::Result<()> {
        self.sensitivity_lists = Vec::new();

        // clear all previously compiled handlers and their data
        let clear: Function = self.lua.globals().get("clear").unwrap();
        clear.call::<_, ()>(())?;
        drop(clear);

        for display in displays {
            self.register_single(display)?;
        }

        Ok(())
    }

    fn register_single(&mut self, display: Display) -> mlua::Result<()> {
        for (code, pos) in display.parts {
            let compile_sandboxed: Function = self.lua.globals().get("compile_sandboxed").unwrap();

            // TODO pass Position and Duration to Lua env
            compile_sandboxed.call::<(String, String), ()>((display.name.clone(), code))?;
        }
        self.sensitivity_lists.push((display.name, display.sensitivity_list));

        Ok(())
    }

    pub fn update(&mut self, plugins: &Vec<(String, HashMap<String, serde_json::Value>)>, interval: Duration) -> mlua::Result<Vec<Operation>> {
        let mut changed = Vec::<String>::new();

        /*
        set value:
            plugin_name.value_name = value
            EVENTS.plugin_name.value_name.notify(value)
         */

        for (name, values) in plugins {
            for (key, value) in values {
                self.set_value(name, key, value)?;

                let key = format!("{}.{}", name, key);
                changed.push(key);
            }
        }

        return Ok(Vec::new());
    }

    fn set_value(&self, table_name: &String, value_name: &String, value: &serde_json::Value) -> mlua::Result<()> {
        let lua = &self.lua;

        let table: Table = lua.globals().get(table_name.clone())?;
        table.set(value_name.clone(), Self::json_to_lua(lua, value)?)?;

        Ok(())
    }

    fn json_to_lua<'a>(lua: &'a Lua, json: &serde_json::Value) -> mlua::Result<Value<'a>> {
        match json {
            serde_json::Value::Null => Ok(Value::Nil),
            serde_json::Value::Bool(val) => {
                match val {
                    true => Ok(Value::Boolean(true)),
                    false => Ok(Value::Boolean(false)),
                }
            }
            serde_json::Value::Number(number) => {
                if let Some(number) = number.as_i64() {
                    return Ok(Value::Integer(number as mlua::Integer));
                }
                let number = number.as_f64().unwrap();
                Ok(Value::Number(number as mlua::Number))
            }
            serde_json::Value::String(string) => {
                let string = lua.create_string(string)?;
                Ok(Value::String(string))
            }
            serde_json::Value::Array(array) => {
                let table = lua.create_table()?;
                for value in array {
                    table.push(Self::json_to_lua(lua, value)?)?;
                }
                Ok(Value::Table(table))
            }
            serde_json::Value::Object(map) => {
                let table = lua.create_table()?;
                for (key, value) in map {
                    table.set(key.clone(), Self::json_to_lua(lua, value)?)?;
                }
                Ok(Value::Table(table))
            }
        }
    }

    fn load_sandbox(lua: &Lua) {
        static SANDBOX_ENV: &str = include_str!("sandbox_env.lua");

        lua.load(SANDBOX_ENV).exec().unwrap();
    }
}