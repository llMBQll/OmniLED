use log::error;
use rdev::{listen, EventType, Key};

use crate::events::event_queue::{Event, EventQueue};

pub fn process_events() {
    if let Err(err) = listen(|event| {
        let event_queue = EventQueue::instance();
        match event.event_type {
            EventType::KeyPress(key) => {
                let mut event_queue = event_queue.lock().unwrap();
                event_queue.push(Event::Keyboard(KeyboardEvent {
                    key,
                    event_type: KeyboardEventEventType::Press,
                }));
            }
            EventType::KeyRelease(key) => {
                let mut event_queue = event_queue.lock().unwrap();
                event_queue.push(Event::Keyboard(KeyboardEvent {
                    key,
                    event_type: KeyboardEventEventType::Release,
                }));
            }
            _ => {}
        }
    }) {
        error!("Keyboard event handler failed: {:?}", err);
    }
}

pub enum KeyboardEventEventType {
    Press,
    Release,
}

pub struct KeyboardEvent {
    pub key: Key,
    pub event_type: KeyboardEventEventType,
}
