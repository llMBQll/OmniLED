use mlua::{Function, Lua, UserData, UserDataMethods, Value};
use omni_led_derive::UniqueUserData;

use crate::common::user_data::UniqueUserData;
use crate::events::event_queue::{Event, EventQueue};

#[derive(UniqueUserData)]
pub struct Events;

impl Events {
    pub fn load(lua: &Lua) {
        Self::set_unique(lua, Self);
    }

    pub fn register(event: String, on_match: Function) {
        EventQueue::instance()
            .lock()
            .unwrap()
            .push_front(Event::Register(RegisterEvent { event, on_match }));
    }

    pub fn send(event: String, value: Value) {
        EventQueue::instance()
            .lock()
            .unwrap()
            .push(Event::Script(ScriptEvent { event, value }));
    }
}

impl UserData for Events {
    fn add_methods<'lua, M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method(
            "register",
            |_lua, _events, (event, on_match): (String, Function)| {
                Events::register(event, on_match);
                Ok(())
            },
        );

        methods.add_method("send", |_lua, _events, (event, value): (String, Value)| {
            Events::send(event, value);
            Ok(())
        });
    }
}

pub struct RegisterEvent {
    pub event: String,
    pub on_match: Function,
}

// SAFETY this event will always be created and read on the Lua thread
unsafe impl Send for RegisterEvent {}

pub struct ScriptEvent {
    pub event: String,
    pub value: Value,
}

// SAFETY this event will always be created and read on the Lua thread
unsafe impl Send for ScriptEvent {}
