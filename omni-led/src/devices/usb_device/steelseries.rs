use mlua::{Lua, Table};

use crate::devices::device::{Buffer, MemoryLayout};

pub fn load_common_functions(lua: &Lua, env: &Table) {
    let transform_selector = lua
        .create_function(|lua, layout: MemoryLayout| {
            let function: fn(&Lua, Buffer) -> mlua::Result<Vec<u8>> = match layout {
                MemoryLayout::BitPerPixel => steelseries_transform,
                MemoryLayout::BitPerPixelVertical => steelseries2_transform,
                MemoryLayout::BytePerPixel => |_, _| {
                    Err(mlua::Error::runtime(
                        "Not supported for SteelSeries devices",
                    ))
                },
            };
            lua.create_function(function)
        })
        .unwrap();

    env.set("transform_steelseries_layout", transform_selector)
        .unwrap();
}

fn steelseries_transform(_lua: &Lua, buffer: Buffer) -> mlua::Result<Vec<u8>> {
    let slice = buffer.bytes();

    let mut bytes = Vec::with_capacity(slice.len() + 2);
    bytes.push(0x61);
    bytes.extend_from_slice(slice);
    bytes.push(0x00);
    Ok(bytes)
}

fn steelseries2_transform(_lua: &Lua, buffer: Buffer) -> mlua::Result<Vec<u8>> {
    let slice = buffer.bytes();

    let mut bytes = Vec::with_capacity(slice.len() + 1);
    bytes.push(0x0a);
    bytes.extend_from_slice(slice);
    Ok(bytes)
}
