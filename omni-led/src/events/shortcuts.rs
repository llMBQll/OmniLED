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

use device_query::Keycode;
use log::{error, warn};
use mlua::{Function, Lua, UserData, UserDataMethods};
use omni_led_derive::UniqueUserData;
use regex::Regex;
use std::str::FromStr;

use crate::common::user_data::{UniqueUserData, UserDataRef};
use crate::settings::settings::Settings;

#[derive(UniqueUserData)]
pub struct Shortcuts {
    shortcuts: Vec<ShortcutEntry>,
    delay: usize,
    rate: usize,
    current_tick: usize,
}

impl Shortcuts {
    pub fn load(lua: &Lua) {
        let settings = UserDataRef::<Settings>::load(lua);
        let delay = settings.get().keyboard_ticks_repeat_delay;
        let rate = settings.get().keyboard_ticks_repeat_rate;

        Self::set_unique(
            lua,
            Self {
                shortcuts: Vec::new(),
                delay,
                rate,
                current_tick: 0,
            },
        );
    }

    pub fn process_key(&mut self, _lua: &Lua, key_name: &str, action: &str) -> mlua::Result<()> {
        for entry in self.shortcuts.iter_mut() {
            let position = match entry.keys.iter().position(|x| x.key == key_name) {
                Some(position) => position,
                None => continue,
            };

            entry.keys[position].pressed = action == "Pressed";
            let all_pressed = entry.keys.iter().all(|x| x.pressed);

            let press = all_pressed && !entry.last_all_pressed;
            let hold = all_pressed && entry.last_all_pressed;
            let required_ticks = match entry.hold_updates {
                0 => self.delay,
                _ => self.rate,
            };
            let delta_ticks = self.current_tick - entry.last_update_tick;
            let update = (self.current_tick != entry.last_update_tick)
                && (press || (hold && delta_ticks >= required_ticks));

            if update {
                entry.last_update_tick = self.current_tick;
                entry.on_match.call::<()>(())?;

                if hold {
                    entry.hold_updates += 1;
                }
            }

            if !hold {
                entry.hold_updates = 0;
            }

            entry.last_all_pressed = all_pressed;
        }
        Ok(())
    }

    pub fn register(&mut self, mut keys: Vec<String>, on_match: Function) -> mlua::Result<()> {
        let pattern = Regex::new(r"^KEY\((.*)\)$").unwrap();

        keys.sort();
        keys.dedup();

        let mut error_found = false;
        let sorted = keys
            .into_iter()
            .filter_map(|key| match pattern.captures(&key) {
                Some(captures) => {
                    let content = captures.get(1).unwrap().as_str();
                    if let Err(_) = Keycode::from_str(content) {
                        warn!(
                            "Failed to parse keycode '{}', this is not always an error",
                            content
                        );
                    }

                    Some(KeyState {
                        key,
                        pressed: false,
                    })
                }
                None => {
                    error!("String '{}' does not match pattern 'KEY(Keycode)'", key);
                    error_found = true;
                    None
                }
            })
            .collect();

        if error_found {
            return Err(mlua::Error::RuntimeError(
                "Failed to parse some of the provided keycodes".to_string(),
            ));
        }

        self.shortcuts.push(ShortcutEntry {
            keys: sorted,
            on_match,
            last_all_pressed: false,
            last_update_tick: 0,
            hold_updates: 0,
        });

        Ok(())
    }

    pub fn update(&mut self) {
        self.current_tick += 1;
    }
}

impl UserData for Shortcuts {
    fn add_methods<'lua, M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut(
            "register",
            |_lua, this, (keys, on_match): (Vec<String>, Function)| this.register(keys, on_match),
        );
    }
}

struct ShortcutEntry {
    keys: Vec<KeyState>,
    on_match: Function,
    last_all_pressed: bool,
    last_update_tick: usize,
    hold_updates: usize,
}

struct KeyState {
    key: String,
    pressed: bool,
}
