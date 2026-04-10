use log::warn;
use mlua::{ErrorContext, FromLua, Function, Lua, UserData, UserDataMethods, Value};
use omni_led_api::types::{Field, field};
use omni_led_derive::UniqueUserData;
use regex::Regex;

use crate::common::user_data::UniqueUserData;
use crate::events::event_queue::Event;
use crate::events::proto_to_lua::{get_cleanup_entries_metatable, proto_to_lua_value};
use crate::keyboard::keyboard::{KeyboardEvent, KeyboardEventEventType};

#[derive(Clone, Copy, PartialEq)]
pub struct EventHandle(usize);

impl UserData for EventHandle {}

impl FromLua for EventHandle {
    fn from_lua(value: Value, _lua: &Lua) -> mlua::Result<Self> {
        match value {
            Value::UserData(user_data) => user_data.borrow::<Self>().map(|k| k.clone()),
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(),
                to: "EventHandle".to_string(),
                message: Some("Expected EventHandle object".to_string()),
            }),
        }
    }
}

pub enum EventKey {
    Regex(Regex),
    String(String),
}

impl EventKey {
    fn matches(&self, event: &str) -> bool {
        match self {
            EventKey::Regex(regex) => regex.is_match(event),
            EventKey::String(string) => string == event,
        }
    }
}

impl From<String> for EventKey {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<Regex> for EventKey {
    fn from(value: Regex) -> Self {
        Self::Regex(value)
    }
}

struct EventEntry {
    key: EventKey,
    on_match: Function,
    handle: EventHandle,
    persistent: bool,
}

#[derive(UniqueUserData)]
pub struct Events {
    entries: Vec<EventEntry>,
    counter: usize,
}

impl Events {
    pub fn load(lua: &Lua) {
        Self::set_unique(
            lua,
            Self {
                entries: Vec::new(),
                counter: 0,
            },
        );
    }

    pub fn register<K: Into<EventKey>>(
        &mut self,
        key: K,
        on_match: Function,
        persistent: bool,
    ) -> EventHandle {
        self.counter += 1;
        let handle = EventHandle(self.counter);

        self.entries.push(EventEntry {
            key: key.into(),
            on_match,
            persistent,
            handle,
        });

        handle
    }

    pub fn unregister(&mut self, handle: EventHandle) -> mlua::Result<()> {
        match self.entries.iter().position(|entry| entry.handle == handle) {
            Some(index) => {
                self.entries.remove(index);
            }
            None => {
                warn!("Failed to unregister event. Key {} not found", handle.0);
            }
        };
        Ok(())
    }

    pub fn clear_non_persistent(&mut self) {
        self.entries.retain(|entry| entry.persistent);
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

impl UserData for Events {
    fn add_methods<'lua, M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut(
            "register",
            |_lua, this, (event, on_match): (String, Function)| {
                // Registering from user scripts must never be persistent to avoid issues on reloads
                Ok(this.register(event, on_match, false))
            },
        );

        methods.add_method_mut(
            "register_regex",
            |_lua, this, (event, on_match): (String, Function)| {
                // Registering from user scripts must never be persistent to avoid issues on reloads
                let regex = Regex::new(&event).map_err(mlua::Error::external)?;
                Ok(this.register(regex, on_match, false))
            },
        );

        methods.add_method_mut("unregister", |_lua, this, key: EventHandle| {
            this.unregister(key)
        });
    }
}
