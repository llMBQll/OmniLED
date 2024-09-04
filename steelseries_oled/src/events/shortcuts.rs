use crate::common::scoped_value::ScopedValue;
use crate::common::user_data::UserDataIdentifier;
use device_query::Keycode;
use log::{error, warn};
use mlua::{Function, Lua, OwnedFunction, UserData, UserDataMethods};
use regex::Regex;
use std::str::FromStr;
use std::time::{Duration, Instant};

pub struct Shortcuts {
    shortcuts: Vec<ShortcutEntry>,
}

impl Shortcuts {
    pub fn load(lua: &Lua) -> ScopedValue {
        ScopedValue::new(
            lua,
            Self::identifier(),
            Self {
                shortcuts: Vec::new(),
            },
        )
    }

    pub fn process_key(&mut self, _lua: &Lua, key_name: &str, action: &str) -> mlua::Result<()> {
        for entry in self.shortcuts.iter_mut() {
            let position = match entry.keys.iter_mut().position(|x| x.key == key_name) {
                Some(position) => position,
                None => continue,
            };

            // calculate all_pressed BEFORE updating the state
            // TODO shouldn't this be the other way around?
            let all_pressed = entry.keys.iter().all(|x| x.pressed);

            entry.keys[position].pressed = action == "Pressed";

            // This disallows updates when someone is repeatedly pressing keys faster than
            // the specified timeout, good enough for now
            // TODO add option to customize retrigger delay and do not discard quick presses
            if all_pressed && Instant::now() - entry.last_update > Duration::from_millis(175) {
                entry.on_match.call::<_, ()>(())?;
                entry.last_update = Instant::now();
            }
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
            on_match: on_match.into_owned(),
            last_update: Instant::now(),
        });

        Ok(())
    }
}

impl UserData for Shortcuts {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut(
            "register",
            |_lua, this, (keys, on_match): (Vec<String>, Function)| this.register(keys, on_match),
        );
    }
}

impl UserDataIdentifier for Shortcuts {
    fn identifier() -> &'static str {
        "SHORTCUTS"
    }
}

struct ShortcutEntry {
    keys: Vec<KeyState>,
    on_match: OwnedFunction,
    last_update: Instant,
}

struct KeyState {
    key: String,
    pressed: bool,
}
