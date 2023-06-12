use std::collections::HashMap;
use std::ffi::c_int;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use mlua::{chunk, Function, Lua, Table, TableExt, UserData, Value};
use tokio::time::Instant;

use crate::settings::settings::Settings;

pub fn load_update_handler(lua: &Lua) -> Arc<Mutex<UpdateHandler>> {
    const UPDATE_HANDLER_SRC: &str = include_str!("update_handler.lua");

    lua.load(UPDATE_HANDLER_SRC).exec().unwrap();
    let update_handler = Arc::new(Mutex::new(UpdateHandler::new()));
    let table: Table = lua.globals().get("UPDATE_HANDLER").unwrap();
    table.set("rust_object", Arc::clone(&update_handler)).unwrap();

    update_handler
}

type Data = (String, HashMap<String, serde_json::Value>);

pub struct UpdateHandler {
    queue: Vec<Data>,
}

impl UpdateHandler {
    pub fn new() -> Self {
        Self {
            queue: Vec::new(),
        }
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
            let interval_integer = Settings::get(lua, "update_interval").unwrap();
            let interval = Duration::from_millis(interval_integer);
            let update_handler: Arc<Mutex<UpdateHandler>> = lua.load(chunk! { UPDATE_HANDLER.rust_object }).eval().unwrap();
            let lua_update_handler: Table = lua.globals().get("UPDATE_HANDLER").unwrap();

            while running.load(Ordering::Relaxed) {
                let begin = Instant::now();

                let data = update_handler.lock().unwrap().get_data();
                for (application, variables) in data {
                    for (name, value) in variables {
                        let value = match json_to_lua_value(lua, value) {
                            Ok(value) => value,
                            Err(_) => {
                                // TODO log error
                                continue;
                            }
                        };

                        // TODO error handling
                        lua_update_handler.call_method::<_, _, ()>("send_value", (application.clone(), name, value)).unwrap();
                    }
                }
                // TODO error handling
                lua_update_handler.call_method::<_, _, ()>("update", interval_integer).unwrap();

                let end = Instant::now();
                let update_duration = end - begin;
                // println!("Update took {} us", update_duration.as_micros());
                tokio::time::sleep(interval.saturating_sub(update_duration)).await;
            }
            Ok(())
        }).unwrap()
    }
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
            let table = lua.create_table_with_capacity(size as c_int, 0)?;
            for value in array {
                table.push(json_to_lua_value(lua, value)?)?;
            }
            Ok(Value::Table(table))
        }
        serde_json::Value::Object(map) => {
            let size = map.len();
            let table = lua.create_table_with_capacity(0, size as c_int)?;
            for (key, value) in map {
                table.set(key.clone(), json_to_lua_value(lua, value)?)?;
            }
            Ok(Value::Table(table))
        }
    }
}

impl UserData for UpdateHandler {}