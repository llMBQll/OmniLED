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

use lazy_static::lazy_static;
use omni_led_api::types::Field;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::keyboard::keyboard::KeyboardEvent;

type ApplicationEvent = (String, HashMap<String, Field>);

pub enum Event {
    Application(ApplicationEvent),
    Keyboard(KeyboardEvent),
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

    pub fn get_events(&mut self) -> Vec<Event> {
        let mut events: Vec<Event> = self.get_default_event_queue();
        std::mem::swap(&mut events, &mut self.queue);
        events
    }

    fn get_default_event_queue(&mut self) -> Vec<Event> {
        // TODO find a better way to register meta events

        let mut map = HashMap::new();
        map.insert("Update".to_string(), self.counter.into());
        self.counter += 1;

        vec![Event::Application(("OMNILED".to_string(), map))]
    }
}
