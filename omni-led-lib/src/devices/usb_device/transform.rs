use mlua::{ErrorContext, FromLua, Lua, Table};
use omni_led_derive::FromLuaValue;

use crate::devices::device::Buffer;

#[derive(Clone, FromLuaValue)]
struct ExtraBytes {
    #[mlua(default)]
    prepend: Vec<u8>,
    #[mlua(default)]
    append: Vec<u8>,
}

pub fn load_common_functions(lua: &Lua, env: &Table) {
    let transform_data = lua
        .create_function(|lua, extra: ExtraBytes| {
            let extra_total = extra.prepend.len() + extra.append.len();

            lua.create_function(move |_, buffer: Buffer| {
                let slice = buffer.bytes();

                let mut bytes = Vec::with_capacity(slice.len() + extra_total);
                bytes.extend_from_slice(&extra.prepend);
                bytes.extend_from_slice(slice);
                bytes.extend_from_slice(&extra.append);
                Ok(bytes)
            })
        })
        .unwrap();

    env.set("transform_data", transform_data).unwrap();
}
