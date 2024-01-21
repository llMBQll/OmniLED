use api::types::field::Field as FieldEntry;
use api::types::Field;
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

pub fn proto_to_lua_value(lua: &Lua, field: Field) -> mlua::Result<Value> {
    match field.field {
        None => Ok(mlua::Nil),
        Some(FieldEntry::FBool(bool)) => Ok(Value::Boolean(bool)),
        Some(FieldEntry::FInteger(integer)) => Ok(Value::Integer(integer)),
        Some(FieldEntry::FFloat(float)) => Ok(Value::Number(float)),
        Some(FieldEntry::FString(string)) => {
            let string = lua.create_string(string)?;
            Ok(Value::String(string))
        }
        Some(FieldEntry::FArray(array)) => {
            let size = array.items.len();
            let table = lua.create_table_with_capacity(size, 0)?;
            for value in array.items {
                table.push(proto_to_lua_value(lua, value)?)?;
            }
            Ok(Value::Table(table))
        }
        Some(FieldEntry::FTable(map)) => {
            let size = map.items.len();
            let table = lua.create_table_with_capacity(0, size)?;
            for (key, value) in map.items {
                table.set(key, proto_to_lua_value(lua, value)?)?;
            }
            Ok(Value::Table(table))
        }
        Some(FieldEntry::FBytes(_)) => {
            todo!("Implement Bytes object conversion")
            // let non_utf_string = unsafe { String::from_utf8_unchecked(bytes.clone()) };
            // let string = lua.create_string(non_utf_string)?;
            // Ok(Value::String(string))
        }
        Some(FieldEntry::FImage(_)) => {
            todo!("Implement Image object conversion")
        }
    }
}
