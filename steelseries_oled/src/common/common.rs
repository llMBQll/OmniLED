use mlua::{Lua, ObjectLike, Table, Value};
use oled_api::field::Field as FieldEntry;
use oled_api::Field;

use crate::script_handler::script_data_types::{OledImage, Size};

#[macro_export]
macro_rules! create_table {
    ($lua:ident, $values:tt) => {
        $lua.load(chunk! { $values }).eval::<mlua::Table>().unwrap()
    };
}

#[macro_export]
macro_rules! create_table_with_defaults {
    ($lua:ident, $values:tt) => {
        {
            let dump_fn = $lua.create_function(|_, value: mlua::Value| {
                let string = format!("{:#?}", value);
                Ok(string)
            }).unwrap();

            let round_fn = $lua.create_function(|_, value: f64| {
                let value = value.round() as i64;
                Ok(value)
            }).unwrap();

            $lua.load(chunk! {
                new_table = $values
                new_table["dump"] = $dump_fn
                new_table["ipairs"] = ipairs
                new_table["next"] = next
                new_table["pairs"] = pairs
                new_table["pcall"] = pcall
                new_table["print"] = print
                new_table["tonumber"] = tonumber
                new_table["tostring"] = tostring
                new_table["type"] = type
                new_table["coroutine"] = { close = coroutine.close, create = coroutine.create, isyieldable = coroutine.isyieldable, resume = coroutine.resume, running = coroutine.running, status = coroutine.status, wrap = coroutine.wrap, yield = coroutine.yield }
                new_table["math"] = { abs = math.abs, acos = math.acos, asin = math.asin, atan = math.atan, atan2 = math.atan2, ceil = math.ceil, cos = math.cos, cosh = math.cosh, deg = math.deg, exp = math.exp, floor = math.floor, fmod = math.fmod, frexp = math.frexp, huge = math.huge, ldexp = math.ldexp, log = math.log, log10 = math.log10, max = math.max, maxinteger = math.maxinteger, min = math.min, mininteger = math.mininteger, modf = math.modf, pi = math.pi, pow = math.pow, rad = math.rad, random = math.random, randomseed = math.randomseed, round = $round_fn, sin = math.sin, sinh = math.sinh, sqrt = math.sqrt, tan = math.tan, tanh = math.tanh, tointeger = math.tointeger, type = math.type, ult = math.ult }
                new_table["os"] = { clock = os.clock, date = os.date, difftime = os.difftime, getenv = os.getenv, time = os.time }
                new_table["string"] = { byte = string.byte, char = string.char, dump = string.dump, find = string.find, format = string.format, gmatch = string.gmatch, gsub = string.gsub, len = string.len, lower = string.lower, match = string.match, pack = string.pack, packsize = string.packsize, rep = string.rep, reverse = string.reverse, sub = string.sub, unpack = string.unpack, upper = string.upper }
                new_table["table"] = { concat = table.concat, insert = table.insert, move = table.move, pack = table.pack, remove = table.remove, sort = table.sort, unpack = table.unpack }
                return new_table
            }).eval::<mlua::Table>().unwrap()
        }
    };
}

pub fn exec_file(lua: &Lua, name: &str, env: Table) -> mlua::Result<()> {
    let (func, err): (Value, Value) = lua.globals().call_function("loadfile", (name, "t", env))?;

    match (func, err) {
        (Value::Function(func), Value::Nil) => func.call::<_>(()),
        (_, Value::String(err)) => Err(mlua::Error::runtime(format!(
            "Error when running file {}: {}",
            name,
            err.to_str()?
        ))),
        _ => Err(mlua::Error::runtime(format!(
            "Error when running file {}",
            name
        ))),
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
