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
