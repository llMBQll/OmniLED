use lazy_static::lazy_static;
use omni_led_api::types::Table;
use std::sync::{Arc, Mutex};

use crate::events::events::ScriptEvent;
use crate::events::{event_handle::EventHandle, events::EventEntry};
use crate::keyboard::keyboard::KeyboardEvent;

type ApplicationEvent = (String, Table);

pub enum Event {
    Application(ApplicationEvent),
    Keyboard(KeyboardEvent),
    Register(EventEntry),
    Unregister(EventHandle),
    ReloadScripts,
    Script(ScriptEvent),
}

pub struct EventQueue {
    queue: Vec<Event>,
    front: usize,
    counter: u64,
}

impl EventQueue {
    pub fn instance() -> Arc<Mutex<EventQueue>> {
        lazy_static! {
            static ref UPDATE_HANDLER: Arc<Mutex<EventQueue>> =
                Arc::new(Mutex::new(EventQueue::new()));
        }

        Arc::clone(&*UPDATE_HANDLER)
    }

    pub fn push(&mut self, event: Event) {
        self.queue.push(event);
    }

    pub fn push_front(&mut self, event: Event) {
        self.queue.insert(self.front, event);
        self.front += 1;
    }

    pub fn get_events(&mut self) -> Vec<Event> {
        self.front = 0;
        self.counter += 1;

        let mut events: Vec<Event> = Self::get_default_event_queue(self.counter);
        std::mem::swap(&mut events, &mut self.queue);

        events
    }

    fn new() -> Self {
        Self {
            queue: Self::get_default_event_queue(0),
            front: 0,
            counter: 0,
        }
    }

    fn get_default_event_queue(counter: u64) -> Vec<Event> {
        // TODO find a better way to register meta events

        let mut table = Table::default();
        table.items.insert("Update".to_string(), counter.into());

        vec![Event::Application(("OMNILED".to_string(), table))]
    }
}
