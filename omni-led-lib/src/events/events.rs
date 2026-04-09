use log::error;
use mlua::{Function, Lua, UserData, UserDataMethods, Value};
use omni_led_api::plugin::Plugin;
use omni_led_derive::UniqueUserData;

use crate::common::user_data::UniqueUserData;
use crate::events::event_handle::EventHandle;
use crate::events::event_queue::{Event, EventQueue};
use crate::script_handler::script_data_types::EventKey;

#[derive(UniqueUserData)]
pub struct Events;

impl Events {
    pub fn load(lua: &Lua) {
        Self::set_unique(lua, Self);
    }

    pub fn register(key: EventKey, on_match: Function, persistent: bool) -> EventHandle {
        let handle = EventHandle::new();
        let entry = EventEntry {
            key: key.into(),
            on_match,
            persistent,
            handle: handle.clone(),
        };

        Self::queue_event(Event::Register(entry));

        handle
    }

    pub fn unregister(handle: EventHandle) {
        Self::queue_event(Event::Unregister(handle));
    }

    pub fn reload_scripts() {
        Self::queue_event(Event::ReloadScripts);
    }

    pub fn send(event: String, value: Value) {
        if Plugin::is_valid_identifier(&event) {
            Self::queue_event(Event::Script(ScriptEvent { event, value }));
        } else {
            error!("'{event}' is not a valid event name");
        }
    }

    fn queue_event(event: Event) {
        EventQueue::instance().lock().unwrap().push_front(event);
    }
}

impl UserData for Events {
    fn add_methods<'lua, M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method(
            "register",
            |_lua, _this, (key, on_match): (EventKey, Function)| {
                // Registering from user scripts must never be persistent to avoid issues on reloads
                Ok(Self::register(key, on_match, false))
            },
        );

        methods.add_method("unregister", |_lua, _this, handle: EventHandle| {
            Ok(Self::unregister(handle))
        });

        methods.add_method("send", |_lua, _this, (event, value): (String, Value)| {
            Ok(Self::send(event, value))
        });
    }
}

pub struct EventEntry {
    pub key: EventKey,
    pub on_match: Function,
    pub handle: EventHandle,
    pub persistent: bool,
}

// SAFETY: This struct will always be created and read from lua interpreter thread
unsafe impl Send for EventEntry {}

pub struct ScriptEvent {
    pub event: String,
    pub value: Value,
}

// SAFETY: This struct will always be created and read from lua interpreter thread
unsafe impl Send for ScriptEvent {}
