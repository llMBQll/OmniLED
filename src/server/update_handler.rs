use lazy_static::lazy_static;
use mlua::{Function, Lua, Table, TableExt, UserData, Value};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tokio::time::Instant;

use crate::settings::settings::NewSettings;

pub fn load_update_handler(lua: &Lua) -> Arc<Mutex<UpdateHandler>> {
    const UPDATE_HANDLER_SRC: &str = include_str!("update_handler.lua");
    lua.load(UPDATE_HANDLER_SRC).exec().unwrap();

    get_update_handler()
}

fn get_update_handler() -> Arc<Mutex<UpdateHandler>> {
    lazy_static! {
        static ref UPDATE_HANDLER: Arc<Mutex<UpdateHandler>> =
            Arc::new(Mutex::new(UpdateHandler::new()));
    }

    Arc::clone(&*UPDATE_HANDLER)
}

type Data = (String, HashMap<String, serde_json::Value>);

pub struct UpdateHandler {
    queue: Vec<Data>,
}

impl UpdateHandler {
    pub fn new() -> Self {
        Self { queue: Vec::new() }
    }

    pub fn push(&mut self, data: Data) {
        self.queue.push(data);
    }

    pub fn get_data(&mut self) -> Vec<Data> {
        let mut data: Vec<Data> = Vec::new();
        std::mem::swap(&mut data, &mut self.queue);
        data
    }

    pub fn make_runner<'a>(lua: &'a Lua, running: &'static AtomicBool) -> Function<'a> {
        lua.create_async_function::<(), (), _, _>(|lua, _| async {
            let interval = NewSettings::get().update_interval;
            let interval_integer = interval.as_millis() as u64;
            let update_handler = get_update_handler();
            let lua_update_handler: Table = lua.globals().get("UPDATE_HANDLER").unwrap();

            while running.load(Ordering::Relaxed) {
                let begin = Instant::now();

                let data = update_handler.lock().unwrap().get_data();
                for (application, variables) in data {
                    for (name, value) in variables {
                        let value = match Self::json_to_lua_value(lua, value) {
                            Ok(value) => value,
                            Err(_) => {
                                // TODO log error
                                continue;
                            }
                        };

                        // TODO error handling
                        lua_update_handler
                            .call_method::<_, ()>("send_value", (application.clone(), name, value))
                            .unwrap();
                    }
                }
                // TODO error handling
                lua_update_handler
                    .call_method::<_, ()>("update", interval_integer)
                    .unwrap();

                let end = Instant::now();
                let update_duration = end - begin;
                // println!("Update took {} us", update_duration.as_micros());
                tokio::time::sleep(interval.saturating_sub(update_duration)).await;
            }
            Ok(())
        })
        .unwrap()
    }

    fn json_to_lua_value(lua: &Lua, json_value: serde_json::Value) -> mlua::Result<Value> {
        match json_value {
            serde_json::Value::Null => Ok(mlua::Nil),
            serde_json::Value::Bool(bool) => Ok(Value::Boolean(bool)),
            serde_json::Value::Number(number) => {
                if let Some(integer) = number.as_i64() {
                    return Ok(Value::Integer(integer));
                }
                Ok(Value::Number(number.as_f64().unwrap()))
            }
            serde_json::Value::String(string) => {
                let string = lua.create_string(&string)?;
                Ok(Value::String(string))
            }
            serde_json::Value::Array(array) => {
                let size = array.len();
                let table = lua.create_table_with_capacity(size, 0)?;
                for value in array {
                    table.push(Self::json_to_lua_value(lua, value)?)?;
                }
                Ok(Value::Table(table))
            }
            serde_json::Value::Object(map) => {
                let size = map.len();
                let table = lua.create_table_with_capacity(0, size)?;
                for (key, value) in map {
                    table.set(key.clone(), Self::json_to_lua_value(lua, value)?)?;
                }
                Ok(Value::Table(table))
            }
        }
    }
}

impl UserData for UpdateHandler {}
