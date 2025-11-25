use lazy_static::lazy_static;
use omni_led_api::types::Table;
use std::sync::{Arc, Mutex};

use crate::events::events::RegisterEvent;
use crate::keyboard::keyboard::KeyboardEvent;

type ApplicationEvent = (String, Table);

pub enum Event {
    Application(ApplicationEvent),
    Keyboard(KeyboardEvent),
    Register(RegisterEvent),
}

pub struct EventQueue {
    queue: Vec<Event>,
    counter: u64,
}

impl EventQueue {
    pub fn instance() -> Arc<Mutex<EventQueue>> {
        lazy_static! {
            static ref UPDATE_HANDLER: Arc<Mutex<EventQueue>> = Arc::new(Mutex::new(EventQueue {
                queue: Vec::new(),
                counter: 0
            }));
        }

        Arc::clone(&*UPDATE_HANDLER)
    }

    pub fn push(&mut self, event: Event) {
        self.queue.push(event);
    }

    pub fn push_front(&mut self, event: Event) {
        self.queue.insert(0, event);
    }

    pub fn get_events(&mut self) -> Vec<Event> {
        let mut events: Vec<Event> = self.get_default_event_queue();
        std::mem::swap(&mut events, &mut self.queue);
        events
    }

    fn get_default_event_queue(&mut self) -> Vec<Event> {
        // TODO find a better way to register meta events

        let mut table = Table::default();
        table
            .items
            .insert("Update".to_string(), self.counter.into());
        self.counter += 1;

        vec![Event::Application(("OMNILED".to_string(), table))]
    }
}
