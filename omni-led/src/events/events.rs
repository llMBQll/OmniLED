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

    pub fn dispatch(&self, event: &str, value: &Value) -> mlua::Result<()> {
        for entry in &self.entries {
            if entry.event == event {
                entry
                    .on_match
                    .call::<()>((event.to_string(), value.clone(), self.current_tick))?;
            }
        }
        Ok(())
    }

    pub fn register(&mut self, event: String, on_match: Function) -> mlua::Result<()> {
        self.entries.push(EventEntry { event, on_match });

        Ok(())
    }

    pub fn update(&mut self) {
        self.current_tick += 1;
    }
}

impl UserData for Events {
    fn add_methods<'lua, M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut(
            "register",
            |_lua, this, (event, on_match): (String, Function)| this.register(event, on_match),
        );
    }
}

struct EventEntry {
    event: String,
    on_match: Function,
}
