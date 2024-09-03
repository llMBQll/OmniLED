use mlua::{Function, Lua, OwnedFunction, UserData, UserDataMethods};
use std::time::{Duration, Instant};

use crate::common::scoped_value::ScopedValue;
use crate::common::user_data::UserDataIdentifier;

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

    pub fn process_key(&mut self, lua: &Lua, key_name: &str, action: &str) -> mlua::Result<()> {
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
            if all_pressed && Instant::now() - entry.last_update > Duration::from_millis(175) {
                entry.on_match.call::<_, ()>(())?;
                entry.last_update = Instant::now();
            }
        }
        Ok(())
    }

    pub fn register(&mut self, mut keys: Vec<String>, on_match: Function) {
        keys.sort();
        keys.dedup();
        let keys = keys
            .into_iter()
            .map(|key| KeyState {
                key,
                pressed: false,
            })
            .collect();

        self.shortcuts.push(ShortcutEntry {
            keys,
            on_match: on_match.into_owned(),
            last_update: Instant::now(),
        });
    }
}

impl UserData for Shortcuts {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut(
            "register",
            |_lua, this, (keys, on_match): (Vec<String>, Function)| {
                this.register(keys, on_match);
                Ok(())
            },
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
