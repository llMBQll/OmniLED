use mlua::{Function, Lua, OwnedFunction, UserData, UserDataFields, UserDataMethods};

use crate::common::scoped_value::ScopedValue;
use crate::common::user_data::{UserDataIdentifier, UserDataRef};
use crate::script_handler::script_handler::ScriptHandler;

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
        for shortcut in self.shortcuts.iter_mut() {
            let position = match shortcut.keys.iter_mut().position(|x| x.key == key_name) {
                Some(position) => position,
                None => continue,
            };

            // calculate all_pressed BEFORE updating the state
            let all_pressed = shortcut.keys.iter().all(|x| x.pressed);

            shortcut.keys[position].pressed = action == "Pressed";

            if all_pressed {
                shortcut.on_match.call::<(), ()>(()).unwrap();
            }

            if (shortcut.flags & flags::RESET_STATE) != 0 {
                let mut script_handler = UserDataRef::<ScriptHandler>::load(lua);
                script_handler.get_mut().reset();
            }
        }
        Ok(())
    }

    pub fn register(&mut self, mut keys: Vec<String>, on_match: Function, flags: Option<u8>) {
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
            flags: flags.unwrap_or(flags::NO_FLAGS),
        });
    }
}

mod flags {
    pub const NO_FLAGS: u8 = 0b00000000;
    pub const RESET_STATE: u8 = 0b00000001;
}

impl UserData for Shortcuts {
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field("NO_FLAGS", flags::NO_FLAGS);
        fields.add_field("RESET_STATE", flags::RESET_STATE);
    }

    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut(
            "register",
            |_lua, this, (keys, on_match, flags): (Vec<String>, Function, Option<u8>)| {
                this.register(keys, on_match, flags);
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
    flags: u8,
}

struct KeyState {
    key: String,
    pressed: bool,
}
