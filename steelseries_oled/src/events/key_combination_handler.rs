use mlua::{Lua, LuaSerdeExt, UserData, UserDataMethods, Value};

pub struct KeyCombinationHandler {
    combinations: Vec<(String, Vec<String>, Vec<bool>)>,
}

impl KeyCombinationHandler {
    pub fn load(lua: &Lua) {
        lua.globals()
            .set(
                "KEY_COMBINATION_HANDLER",
                KeyCombinationHandler {
                    combinations: Vec::new(),
                },
            )
            .unwrap()
    }
}

impl UserData for KeyCombinationHandler {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut(
            "register_combination",
            |_lua, this, (prefix, mut combination): (String, Vec<String>)| {
                combination.sort();
                combination.dedup();
                let state = combination.iter().map(|_| false).collect();
                this.combinations.push((prefix, combination, state));

                Ok(())
            },
        );

        methods.add_method_mut("handle_key", |lua, this, (event, data): (String, Value)| {
            for (prefix, combination, state) in this.combinations.iter_mut() {
                let position = match combination.iter().position(|x| *x == event) {
                    Some(position) => position,
                    None => continue,
                };
                let data: String = lua.from_value(data.clone()).unwrap();
                let all_pressed = state.iter().all(|x| *x == true);

                state[position] = data == "Pressed";
                if all_pressed {
                    return Ok(Some(prefix.clone()));
                } else {
                    break;
                }
            }

            Ok(None)
        });
    }
}
