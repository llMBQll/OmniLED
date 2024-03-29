use mlua::{Function, Lua, LuaSerdeExt, OwnedFunction, UserData, UserDataMethods, Value};

use crate::common::scoped_value::ScopedValue;

pub struct Shortcuts {
    shortcuts: Vec<ShortcutEntry>,
}

impl Shortcuts {
    pub fn load(lua: &Lua) -> ScopedValue {
        ScopedValue::new(
            lua,
            "SHORTCUTS",
            Self {
                shortcuts: Vec::new(),
            },
        )
    }
}

impl UserData for Shortcuts {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut(
            "register",
            |_lua, this, (mut keys, on_match): (Vec<String>, Function)| {
                keys.sort();
                keys.dedup();
                let keys = keys
                    .into_iter()
                    .map(|key| KeyState {
                        key,
                        pressed: false,
                    })
                    .collect();

                this.shortcuts.push(ShortcutEntry {
                    keys,
                    on_match: on_match.into_owned(),
                });

                Ok(())
            },
        );

        methods.add_method_mut(
            "process_key",
            |lua, this, (event, event_data): (String, Value)| {
                for shortcut in this.shortcuts.iter_mut() {
                    let position = match shortcut.keys.iter_mut().position(|x| x.key == event) {
                        Some(position) => position,
                        None => continue,
                    };

                    // calculate all_pressed BEFORE updating the state
                    let all_pressed = shortcut.keys.iter().all(|x| x.pressed);

                    let data: String = lua.from_value(event_data.clone()).unwrap();
                    shortcut.keys[position].pressed = data == "Pressed";

                    if all_pressed {
                        shortcut.on_match.call::<(), ()>(()).unwrap();
                    }
                }
                Ok(())
            },
        )
    }
}

struct ShortcutEntry {
    keys: Vec<KeyState>,
    on_match: OwnedFunction,
}

struct KeyState {
    key: String,
    pressed: bool,
}
