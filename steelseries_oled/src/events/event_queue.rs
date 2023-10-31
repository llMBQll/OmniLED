use lazy_static::lazy_static;
use mlua::{Lua, UserData};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::keyboard::keyboard::KeyboardEvent;

type ApplicationEvent = (String, HashMap<String, serde_json::Value>);

pub enum Event {
    Application(ApplicationEvent),
    Keyboard(KeyboardEvent),
}

pub struct EventQueue {
    queue: Vec<Event>,
}

impl EventQueue {
    pub fn load(lua: &Lua) {
        const EVENT_HANDLER_SRC: &str = include_str!("event_handler.lua");
        lua.load(EVENT_HANDLER_SRC).exec().unwrap();
    }

    pub fn instance() -> Arc<Mutex<EventQueue>> {
        lazy_static! {
            static ref UPDATE_HANDLER: Arc<Mutex<EventQueue>> =
                Arc::new(Mutex::new(EventQueue { queue: Vec::new() }));
        }

        Arc::clone(&*UPDATE_HANDLER)
    }

    pub fn push(&mut self, event: Event) {
        self.queue.push(event);
    }

    pub fn get_events(&mut self) -> Vec<Event> {
        let mut events: Vec<Event> = Vec::new();
        std::mem::swap(&mut events, &mut self.queue);
        events
    }
}

impl UserData for EventQueue {}
