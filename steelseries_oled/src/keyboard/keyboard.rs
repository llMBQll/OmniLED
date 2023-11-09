use device_query::{DeviceQuery, DeviceState, Keycode};
use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

use crate::events::event_queue::{Event, EventQueue};

pub fn process_events(running: &AtomicBool) {
    let device_state = DeviceState::new();
    let event_queue = EventQueue::instance();
    let mut previous_state: Vec<Keycode> = Vec::new();

    while running.load(Ordering::Relaxed) {
        let keys = device_state.get_keys();
        {
            let mut guard = event_queue.lock().unwrap();
            for key in keys.iter() {
                if !previous_state.contains(&key) {
                    guard.push(Event::Keyboard(KeyboardEvent {
                        key: key.clone(),
                        event_type: KeyboardEventEventType::Press,
                    }));
                }
            }
            for key in previous_state {
                if !keys.contains(&key) {
                    guard.push(Event::Keyboard(KeyboardEvent {
                        key,
                        event_type: KeyboardEventEventType::Release,
                    }));
                }
            }
        }
        previous_state = keys;

        std::thread::sleep(Duration::from_millis(10));
    }
}

pub enum KeyboardEventEventType {
    Press,
    Release,
}

pub struct KeyboardEvent {
    pub key: Keycode,
    pub event_type: KeyboardEventEventType,
}
