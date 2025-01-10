/*
 * OmniLED is a software for displaying data on various OLED devices.
 * Copyright (C) 2024  Michał Bałabanow <m.balabanow@gmail.com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use mlua::{chunk, ErrorContext, Lua, ObjectLike, Table, Value};
use omni_led_api::types::field::Field as FieldEntry;
use omni_led_api::types::Field;

use crate::script_handler::script_data_types::{OledImage, Size};

#[macro_export]
macro_rules! create_table {
    ($lua:ident, $values:tt) => {
        $lua.load(chunk! { $values }).eval::<mlua::Table>().unwrap()
    };
}

#[macro_export]
macro_rules! create_table_with_defaults {
    ($lua:ident, $values:tt) => {{
        $lua.load(chunk! {
            new_table = $values
            new_table.assert = assert
            new_table.coroutine = internal.table_copy(coroutine)
            new_table.dump = internal.dump
            new_table.getmetatable = getmetatable
            new_table.ipairs = ipairs
            new_table.math = internal.table_copy(math)
            new_table.math.round = internal.round
            new_table.next = next
            new_table.os = {
                clock = os.clock,
                date = os.date,
                difftime = os.difftime,
                getenv = os.getenv,
                time = os.time
            }
            new_table.pairs = pairs
            new_table.pcall = pcall
            new_table.print = print
            new_table.string = internal.table_copy(string)
            new_table.table = internal.table_copy(table)
            new_table.tonumber = tonumber
            new_table.tostring = tostring
            new_table.type = type
            new_table.utf8 = internal.table_copy(utf8)
            return new_table
        })
        .eval::<mlua::Table>()
        .unwrap()
    }};
}

pub fn load_internal_functions(lua: &Lua) {
    let dump = lua
        .create_function(|_, value: Value| {
            let string = format!("{:#?}", value);
            Ok(string)
        })
        .unwrap();

    let round = lua
        .create_function(|_, value: f64| {
            let value = value.round() as i64;
            Ok(value)
        })
        .unwrap();

    let table_copy = lua
        .create_function(|lua, table: Table| -> mlua::Result<Table> {
            let new_table = lua.create_table().unwrap();
            for pair in table.pairs::<String, Value>() {
                let (key, value) = pair?;
                new_table.set(key, value)?;
            }
            Ok(new_table)
        })
        .unwrap();

    lua.globals()
        .set(
            "internal",
            create_table!(lua, {
                dump = $dump,
                round = $round,
                table_copy = $table_copy
            }),
        )
        .unwrap();
}

pub fn exec_file(lua: &Lua, name: &str, env: Table) -> mlua::Result<()> {
    let (func, err): (Value, Value) = lua.globals().call_function("loadfile", (name, "t", env))?;

    let function = match (func, err) {
        (Value::Function(func), Value::Nil) => Ok(func),
        (_, Value::String(err)) => Err(mlua::Error::runtime(err.to_string_lossy())),
        _ => Err(mlua::Error::runtime("Unknown error")),
    };

    function
        .and_then(|function| function.call(()))
        .map_err(|err| err.with_context(|_| format!("Running '{}'", name)))
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
        Some(FieldEntry::FImage(image)) => {
            let oled_image = OledImage {
                size: Size {
                    width: image.width as usize,
                    height: image.height as usize,
                },
                bytes: image.data,
            };
            let user_data = lua.create_any_userdata(oled_image)?;
            Ok(Value::UserData(user_data))
        }
    }
}
