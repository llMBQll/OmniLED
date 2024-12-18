/*
 * OmniLED is a software for displaying data on various OLED devices.
 * Copyright (C) 2024  Michał Bałabanow <m.balabanow@gmail.com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

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
                guard.push(Event::Keyboard(KeyboardEvent {
                    key: key.clone(),
                    event_type: KeyboardEventEventType::Press,
                }));
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

        std::thread::sleep(Duration::from_millis(25));
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
