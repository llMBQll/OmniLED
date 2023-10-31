use log::error;
use mlua::{Lua, Table, TableExt, Value};

#[macro_export]
macro_rules! create_table {
    ($lua:ident, $values:tt) => {
        $lua.load(chunk! { $values }).eval::<mlua::Table>().unwrap()
    };
}

pub fn exec_file(lua: &Lua, name: &str, env: Table) {
    let (func, err): (Value, Value) = lua
        .globals()
        .call_function("loadfile", (name, "t", env))
        .unwrap();

    match (func, err) {
        (Value::Function(func), Value::Nil) => func.call::<_, ()>(()).unwrap(),
        (_, Value::String(err)) => error!("Error when running file: {}", err.to_str().unwrap()),
        _ => error!("Error when running file"),
    }
}

pub fn json_to_lua_value(lua: &Lua, json_value: serde_json::Value) -> mlua::Result<Value> {
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
                table.push(json_to_lua_value(lua, value)?)?;
            }
            Ok(Value::Table(table))
        }
        serde_json::Value::Object(map) => {
            let size = map.len();
            let table = lua.create_table_with_capacity(0, size)?;
            for (key, value) in map {
                table.set(key.clone(), json_to_lua_value(lua, value)?)?;
            }
            Ok(Value::Table(table))
        }
    }
}
