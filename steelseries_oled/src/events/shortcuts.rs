use mlua::{
    AnyUserData, AnyUserDataExt, Function, Lua, LuaSerdeExt, OwnedFunction, UserData,
    UserDataFields, UserDataMethods, Value,
};

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
            |_lua, this, (mut keys, on_match, flags): (Vec<String>, Function, Option<u8>)| {
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
                    flags: flags.unwrap_or(flags::NO_FLAGS),
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

                    if (shortcut.flags & flags::RESET_STATE) != 0 {
                        let event_handler: AnyUserData = lua.globals().get("SCRIPT_HANDLER")?;
                        event_handler.call_method::<_, ()>("reset", ())?;
                    }
                }
                Ok(())
            },
        );
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
