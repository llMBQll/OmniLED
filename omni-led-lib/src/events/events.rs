use mlua::{ErrorContext, Function, Lua, Table, UserData, UserDataMethods, Value};
use omni_led_api::types::field::Field as FieldEntry;
use omni_led_api::types::{Field, field};
use omni_led_derive::UniqueUserData;
use std::hash::{DefaultHasher, Hash, Hasher};

use crate::common::user_data::UniqueUserData;
use crate::events::event_queue::Event;
use crate::keyboard::keyboard::{KeyboardEvent, KeyboardEventEventType};
use crate::script_handler::script_data_types::ImageData;

#[derive(UniqueUserData)]
pub struct Events {
    entries: Vec<EventEntry>,
}

impl Events {
    pub fn load(lua: &Lua) {
        Self::set_unique(
            lua,
            Self {
                entries: Vec::new(),
            },
        );
    }

    pub fn register(&mut self, event: String, on_match: Function) -> mlua::Result<()> {
        self.entries.push(EventEntry { event, on_match });

        Ok(())
    }

    pub fn dispatch(&self, lua: &Lua, event: Event) -> mlua::Result<()> {
        match event {
            Event::Application((application, value)) => {
                let value = Field {
                    field: Some(field::Field::FTable(value)),
                };
                let value = proto_to_lua_value(&lua, value)
                    .map_err(|err| err.with_context(|_| "Failed to convert protobuf value"))?;

                self.dispatch_application_event(&application, value, None)
            }
            Event::Keyboard(event) => self.dispatch_keyboard_event(lua, event),
        }
    }

    fn dispatch_application_event(
        &self,
        value_name: &str,
        value: Value,
        current_key: Option<&str>,
    ) -> mlua::Result<()> {
        let current_key = match current_key {
            Some(current_key) => format!("{}.{}", current_key, value_name),
            None => value_name.to_string(),
        };

        self.dispatch_event(&current_key, &value)?;

        match value {
            Value::Table(table) => match table.metatable() {
                Some(metatable) => {
                    if !metatable.contains_key(CLEANUP_ENTRIES)? {
                        return Err(mlua::Error::runtime(format!(
                            "Unexpected metatable {:#?}",
                            metatable
                        )));
                    }

                    table.for_each(|key: String, val: Value| {
                        self.dispatch_application_event(&key, val, Some(&current_key))
                    })
                }
                None => Ok(()),
            },
            _ => Ok(()),
        }
    }

    fn dispatch_keyboard_event(&self, lua: &Lua, event: KeyboardEvent) -> mlua::Result<()> {
        let key_name = format!("KEY({})", event.key);
        let action = match event.event_type {
            KeyboardEventEventType::Press => "Pressed",
            KeyboardEventEventType::Release => "Released",
        };
        let action = Value::String(lua.create_string(action)?);

        self.dispatch_event(&key_name, &action)
    }

    fn dispatch_event(&self, event: &str, value: &Value) -> mlua::Result<()> {
        for entry in &self.entries {
            if entry.event == event || entry.event == "*" {
                entry
                    .on_match
                    .call::<()>((event.to_string(), value.clone()))?;
            }
        }
        Ok(())
    }
}

impl UserData for Events {
    fn add_methods<'lua, M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut(
            "register",
            |_lua, this, (event, on_match): (String, Function)| this.register(event, on_match),
        );
    }
}

struct EventEntry {
    event: String,
    on_match: Function,
}

const CLEANUP_ENTRIES: &str = "__cleanup_entries";

pub fn get_cleanup_entries_metatable(table: &Table) -> mlua::Result<Option<Table>> {
    match table.metatable() {
        Some(metatable) => metatable.get(CLEANUP_ENTRIES),
        None => Ok(None),
    }
}

pub fn proto_to_lua_value(lua: &Lua, field: Field) -> mlua::Result<Value> {
    match field.field {
        Some(FieldEntry::FNone(_)) | None => Ok(mlua::Nil),
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
            let cleanup_entries = lua.create_table_with_capacity(0, size)?;

            for (key, value) in map.items {
                match proto_to_lua_value(lua, value)? {
                    Value::Nil => cleanup_entries.set(key, true)?,
                    value => table.set(key, value)?,
                }
            }

            let meta = lua.create_table_with_capacity(0, 1)?;
            meta.set(CLEANUP_ENTRIES, cleanup_entries)?;
            _ = table.set_metatable(Some(meta));

            Ok(Value::Table(table))
        }
        Some(FieldEntry::FImageData(image)) => {
            let hash = hash(&image.data);
            let image_data = ImageData {
                format: image.format().try_into().map_err(mlua::Error::external)?,
                bytes: image.data,
                hash: Some(hash),
            };
            let user_data = lua.create_any_userdata(image_data)?;
            Ok(Value::UserData(user_data))
        }
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
    use omni_led_api::types::None;
    use omni_led_derive::IntoProto;

    #[test]
    fn convert_nil() {
        let lua = Lua::new();
        assert_eq!(
            proto_to_lua_value(&lua, None {}.into()).unwrap(),
            Value::Nil
        )
    }

    #[test]
    fn convert_bool() {
        let lua = Lua::new();
        assert_eq!(
            proto_to_lua_value(&lua, true.into()).unwrap(),
            Value::Boolean(true)
        )
    }

    #[test]
    fn convert_integer() {
        let lua = Lua::new();
        assert_eq!(
            proto_to_lua_value(&lua, 68.into()).unwrap(),
            Value::Integer(68)
        )
    }

    #[test]
    fn convert_float() {
        let lua = Lua::new();
        assert_eq!(
            proto_to_lua_value(&lua, 6.8.into()).unwrap(),
            Value::Number(6.8)
        )
    }

    #[test]
    fn convert_string() {
        let lua = Lua::new();
        let string = "Omegalul";
        assert_eq!(
            proto_to_lua_value(&lua, string.into()).unwrap(),
            Value::String(lua.create_string(string).unwrap())
        )
    }

    #[test]
    fn convert_array() {
        let lua = Lua::new();
        let array = vec![1, 2, 3, 4];

        let result = proto_to_lua_value(&lua, array.clone().into()).unwrap();
        assert!(result.is_table());

        let result = result.as_table().unwrap();
        let cleanup_entries = get_cleanup_entries_metatable(result).unwrap();

        assert!(cleanup_entries.is_none());
        assert_eq!(result.len().unwrap() as usize, array.len());
        result
            .for_each(|lua_index: usize, value: i64| {
                let index = lua_index - 1;
                assert_eq!(value, array[index], "Missmatch at index {}", index);
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn convert_table() {
        #[derive(IntoProto)]
        struct Input {
            a: i64,
            b: String,
            c: Option<bool>,
            d: Option<bool>,
            #[proto(strong_none)]
            e: Option<f64>,
            #[proto(strong_none)]
            f: Option<f64>,
        }

        let lua = Lua::new();
        let input = Input {
            a: 0,
            b: "b".to_string(),
            c: None,
            d: Some(true),
            e: None,
            f: Some(1.23),
        };

        let result = proto_to_lua_value(&lua, input.into()).unwrap();
        assert!(result.is_table());

        let result = result.as_table().unwrap();
        let cleanup_entries = get_cleanup_entries_metatable(result).unwrap();

        assert!(cleanup_entries.is_some());

        let cleanup_entries = cleanup_entries.unwrap();

        assert_eq!(cleanup_entries.pairs::<Value, Value>().count(), 1);
        // We only care that the key is present. As long as it's there, the condition will hold.
        assert_ne!(cleanup_entries.get::<Value>("e").unwrap(), Value::Nil);

        assert_eq!(result.pairs::<Value, Value>().count(), 4);
        assert_eq!(result.get::<i64>("a").unwrap(), 0);
        assert_eq!(result.get::<String>("b").unwrap(), String::from("b"));
        assert_eq!(result.get::<Option<bool>>("d").unwrap(), Some(true));
        assert_eq!(result.get::<Option<f64>>("f").unwrap(), Some(1.23));
    }
}
