use mlua::{ErrorContext, Function, Lua, Value};
use omni_led_api::types::{Field, field};

use crate::common::common::{KEY_VAL_TABLE, proto_to_lua_value};
use crate::events::event_queue::Event;
use crate::events::events::RegisterEvent;
use crate::keyboard::keyboard::{KeyboardEvent, KeyboardEventEventType};

pub struct Dispatcher {
    entries: Vec<EventHandlerEntry>,
}

impl Dispatcher {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn dispatch(&mut self, lua: &Lua, event: Event) -> mlua::Result<()> {
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
            Event::Register(event) => self.handle_register_event(event),
            Event::Script(event) => {
                self.dispatch_application_event(&event.event, event.value, None)
            }
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
                    if !metatable.contains_key(KEY_VAL_TABLE)? {
                        unreachable!("Only key-value tables should have a metatable")
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

    fn handle_register_event(&mut self, register_event: RegisterEvent) -> mlua::Result<()> {
        self.entries.push(EventHandlerEntry {
            event: register_event.event,
            on_match: register_event.on_match,
        });
        Ok(())
    }
}

struct EventHandlerEntry {
    event: String,
    on_match: Function,
}
