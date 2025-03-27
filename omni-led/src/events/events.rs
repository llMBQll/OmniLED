/*
 * OmniLED is a software for displaying data on various OLED devices.
 * Copyright (C) 2025  Michał Bałabanow <m.balabanow@gmail.com>
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

use mlua::{Function, Lua, UserData, UserDataMethods, Value};
use omni_led_derive::UniqueUserData;

use crate::common::user_data::UniqueUserData;

#[derive(UniqueUserData)]
pub struct Events {
    entries: Vec<EventEntry>,
    current_tick: usize,
}

impl Events {
    pub fn load(lua: &Lua) {
        Self::set_unique(
            lua,
            Self {
                entries: Vec::new(),
                current_tick: 0,
            },
        );
    }

    pub fn process_event(&mut self, _lua: &Lua, event: &str, value: &Value) -> mlua::Result<()> {
        for entry in self.entries.iter_mut() {
            if entry.last_trigger_tick == self.current_tick {
                continue;
            }

            match entry.events.iter().position(|e| e.event == event) {
                Some(position) => entry.events[position].triggered = true,
                None => continue,
            };

            let all_triggered = entry.events.iter().all(|x| x.triggered);
            if all_triggered {
                entry.last_trigger_tick = self.current_tick;
                entry.on_match.call::<()>(value.clone())?;
            }
        }
        Ok(())
    }

    pub fn register(&mut self, mut events: Vec<String>, on_match: Function) -> mlua::Result<()> {
        events.sort();
        events.dedup();

        let events = events
            .into_iter()
            .map(|event| EventState {
                event,
                triggered: false,
            })
            .collect();

        self.entries.push(EventEntry {
            events,
            on_match,
            last_trigger_tick: 0,
        });

        Ok(())
    }

    pub fn update(&mut self) {
        self.current_tick += 1;

        for entry in &mut self.entries {
            for event in &mut entry.events {
                event.triggered = false;
            }
        }
    }
}

impl UserData for Events {
    fn add_methods<'lua, M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut(
            "register",
            |_lua, this, (events, on_match): (Vec<String>, Function)| {
                this.register(events, on_match)
            },
        );
    }
}

struct EventEntry {
    events: Vec<EventState>,
    on_match: Function,
    last_trigger_tick: usize,
}

struct EventState {
    event: String,
    triggered: bool,
}
