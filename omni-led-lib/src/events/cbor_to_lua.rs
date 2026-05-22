use ciborium::Value as CborValue;
use mlua::{Lua, Table, Value as LuaValue};
use omni_led_api::types::{Image, Tagged};
use std::hash::{DefaultHasher, Hash, Hasher};

use crate::script_handler::script_data_types::ImageData;

const CLEANUP_ENTRIES: &str = "__cleanup_entries";

pub fn get_cleanup_entries_metatable(table: &Table) -> mlua::Result<Option<Table>> {
    match table.metatable() {
        Some(metatable) => metatable.get(CLEANUP_ENTRIES),
        None => Ok(None),
    }
}

pub fn marked_table(lua: &Lua, table: Table) -> mlua::Result<Table> {
    set_cleanup_entries_metatable(lua, &table, lua.create_table()?)?;
    Ok(table)
}

fn set_cleanup_entries_metatable(lua: &Lua, table: &Table, entries: Table) -> mlua::Result<()> {
    let meta = lua.create_table_with_capacity(0, 1)?;
    meta.set(CLEANUP_ENTRIES, entries)?;
    table.set_metatable(Some(meta))
}

pub fn cbor_to_lua_value(lua: &Lua, value: CborValue) -> mlua::Result<LuaValue> {
    match value {
        CborValue::Integer(integer) => {
            let integer: i64 = integer.try_into().map_err(mlua::Error::external)?;
            Ok(LuaValue::Integer(integer))
        }
        CborValue::Bytes(bytes) => {
            let bytes = lua.create_string(bytes)?;
            Ok(LuaValue::String(bytes))
        }
        CborValue::Float(float) => Ok(LuaValue::Number(float)),
        CborValue::Text(string) => {
            let string = lua.create_string(string)?;
            Ok(LuaValue::String(string))
        }
        CborValue::Bool(bool) => Ok(LuaValue::Boolean(bool)),
        CborValue::Null => Ok(LuaValue::Nil),
        CborValue::Tag(tag, value) => match tag {
            Image::TAG => {
                let image: Image = value.deserialized().map_err(mlua::Error::external)?;

                let hash = hash(&image.bytes);
                let image_data = ImageData {
                    format: image.format,
                    bytes: image.bytes,
                    hash: Some(hash),
                };
                let user_data = lua.create_any_userdata(image_data)?;
                Ok(LuaValue::UserData(user_data))
            }
            other => Err(mlua::Error::runtime(format!("Unexpected tag: {}", other))),
        },
        CborValue::Array(values) => {
            let size = values.len();
            let table = lua.create_table_with_capacity(size, 0)?;
            for value in values {
                table.push(cbor_to_lua_value(lua, value)?)?;
            }
            Ok(LuaValue::Table(table))
        }
        CborValue::Map(items) => {
            let size = items.len();

            let table = lua.create_table_with_capacity(0, size)?;
            let cleanup_entries = lua.create_table_with_capacity(0, size)?;

            for (key, value) in items {
                let key = key
                    .into_text()
                    .map_err(|v| mlua::Error::runtime(format!("Expected text key, got {:?}", v)))?;

                match cbor_to_lua_value(lua, value)? {
                    LuaValue::Nil => cleanup_entries.set(key, true)?,
                    value => table.set(key, value)?,
                }
            }

            set_cleanup_entries_metatable(&lua, &table, cleanup_entries)?;

            Ok(LuaValue::Table(table))
        }
        other => Err(mlua::Error::runtime(format!(
            "Unexpected value: {:?}",
            other
        ))),
    }
}

fn hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ciborium::cbor;

    #[test]
    fn convert_nil() {
        let lua = Lua::new();
        assert_eq!(
            cbor_to_lua_value(&lua, CborValue::Null).unwrap(),
            LuaValue::Nil
        )
    }

    #[test]
    fn convert_bool() {
        let lua = Lua::new();
        assert_eq!(
            cbor_to_lua_value(&lua, true.into()).unwrap(),
            LuaValue::Boolean(true)
        )
    }

    #[test]
    fn convert_integer() {
        let lua = Lua::new();
        assert_eq!(
            cbor_to_lua_value(&lua, 68.into()).unwrap(),
            LuaValue::Integer(68)
        )
    }

    #[test]
    fn convert_float() {
        let lua = Lua::new();
        assert_eq!(
            cbor_to_lua_value(&lua, 6.8.into()).unwrap(),
            LuaValue::Number(6.8)
        )
    }

    #[test]
    fn convert_string() {
        let lua = Lua::new();
        let string = "Omegalul";
        assert_eq!(
            cbor_to_lua_value(&lua, string.into()).unwrap(),
            LuaValue::String(lua.create_string(string).unwrap())
        )
    }

    #[test]
    fn convert_array() {
        let lua = Lua::new();
        let array = vec![1, 2, 3, 4];

        let value = CborValue::Array(array.iter().map(|v| (*v).into()).collect());
        let result = cbor_to_lua_value(&lua, value).unwrap();
        assert!(result.is_table(), "Actually is {}", result.type_name());

        let result = result.as_table().unwrap();
        let cleanup_entries = get_cleanup_entries_metatable(result).unwrap();

        assert!(cleanup_entries.is_none());
        assert_eq!(result.len().unwrap() as usize, array.len());
        result
            .for_each(|lua_index: usize, value: u8| {
                let index = lua_index - 1;
                assert_eq!(value, array[index], "Missmatch at index {}", index);
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn convert_table() {
        let lua = Lua::new();
        let value = cbor!({
            "a" => 0,
            "b" => "b",
            "c" => true,
            "d" => 1.23,
            "e" => null,
        })
        .unwrap();

        let result = cbor_to_lua_value(&lua, value).unwrap();
        assert!(result.is_table());

        let result = result.as_table().unwrap();
        let cleanup_entries = get_cleanup_entries_metatable(result).unwrap();

        assert!(cleanup_entries.is_some());

        let cleanup_entries = cleanup_entries.unwrap();

        assert_eq!(cleanup_entries.pairs::<LuaValue, LuaValue>().count(), 1);
        // We only care that the key is present. As long as it's there, the condition will hold.
        assert_ne!(cleanup_entries.get::<LuaValue>("e").unwrap(), LuaValue::Nil);

        assert_eq!(result.pairs::<LuaValue, LuaValue>().count(), 4);
        assert_eq!(result.get::<i64>("a").unwrap(), 0);
        assert_eq!(result.get::<String>("b").unwrap(), String::from("b"));
        assert_eq!(result.get::<bool>("c").unwrap(), true);
        assert_eq!(result.get::<f64>("d").unwrap(), 1.23);
    }
}
