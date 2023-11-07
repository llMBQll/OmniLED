use device_query::{DeviceQuery, DeviceState, Keycode};
use std::time::Duration;

use crate::events::event_queue::{Event, EventQueue};

pub async fn process_events() {
    let device_state = DeviceState::new();
    let event_queue = EventQueue::instance();
    let mut previous_state: Vec<Keycode> = Vec::new();

    loop {
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

        tokio::time::sleep(Duration::from_millis(10)).await;
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
