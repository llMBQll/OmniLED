use log::warn;
use mlua::{ErrorContext, Lua, Value};
use omni_led_api::types::{Field, field};

use crate::events::event_handle::EventHandle;
use crate::events::event_queue::Event;
use crate::events::events::EventEntry;
use crate::events::proto_to_lua::{get_cleanup_entries_metatable, proto_to_lua_value};
use crate::keyboard::keyboard::{KeyboardEvent, KeyboardEventEventType};

pub struct Dispatcher {
    entries: Vec<EventEntry>,
    counter: usize,
}

impl Dispatcher {
    pub fn load(_lua: &Lua) -> Self {
        Self {
            entries: Vec::new(),
            counter: 0,
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
            Event::Register(event_entry) => self.register(event_entry),
            Event::Unregister(event_handle) => self.unregister(event_handle),
            Event::ClearUserEvents => self.clear_non_persistent(),
            Event::Script(script_event) => {
                self.dispatch_application_event(&script_event.event, script_event.value, None)
            }
        }
    }

    fn register(&mut self, entry: EventEntry) -> mlua::Result<()> {
        self.counter += 1;
        entry.handle.assign_id(self.counter);
        self.entries.push(entry);
        Ok(())
    }

    fn unregister(&mut self, handle: EventHandle) -> mlua::Result<()> {
        match self.entries.iter().position(|entry| entry.handle == handle) {
            Some(index) => {
                self.entries.remove(index);
            }
            None => {
                warn!(
                    "Failed to unregister event. Key {} not found",
                    handle.get_id()
                );
            }
        };
        Ok(())
    }

    fn clear_non_persistent(&mut self) -> mlua::Result<()> {
        self.entries.retain(|entry| entry.persistent);
        Ok(())
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

        if let Value::Table(table) = value {
            if let Some(_) = get_cleanup_entries_metatable(&table)? {
                return table.for_each(|key: String, val: Value| {
                    self.dispatch_application_event(&key, val, Some(&current_key))
                });
            }
        }

        Ok(())
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
            if entry.key.matches(event) {
                entry
                    .on_match
                    .call::<()>((event.to_string(), value.clone()))?;
            }
        }
        Ok(())
    }
}
