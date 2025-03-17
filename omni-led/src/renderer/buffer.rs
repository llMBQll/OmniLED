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

use mlua::{UserData, UserDataMethods};

use crate::devices::device::MemoryRepresentation;
use crate::renderer::bit::{Bit, BitMut};
use crate::script_handler::script_data_types::Modifiers;
use crate::script_handler::script_data_types::{Rectangle, Size};

pub struct Buffer {
    buffer: Box<dyn BufferTrait>,
}

impl Buffer {
    pub fn new(size: Size, memory_representation: MemoryRepresentation) -> Self {
        let buffer: Box<dyn BufferTrait> = match memory_representation {
            MemoryRepresentation::BitPerPixel => Box::new(BitBuffer::new(size)),
            MemoryRepresentation::BytePerPixel => Box::new(ByteBuffer::new(size)),
            MemoryRepresentation::BitPerPixelVertical => Box::new(VerticalBitBuffer::new(size)),
        };

        Self { buffer }
    }

    pub fn set(&mut self, x: isize, y: isize, area: &Rectangle, modifiers: &Modifiers) {
        self.set_value(true, x, y, area, modifiers);
    }

    pub fn reset(&mut self, x: isize, y: isize, area: &Rectangle, modifiers: &Modifiers) {
        self.set_value(false, x, y, area, modifiers);
    }

    pub fn bytes(&self) -> &[u8] {
        self.buffer.bytes().as_slice()
    }

    fn set_value(
        &mut self,
        value: bool,
        x: isize,
        y: isize,
        area: &Rectangle,
        modifiers: &Modifiers,
    ) {
        let (x, y) = match self.translate(x, y, area, modifiers) {
            Some(pos) => pos,
            None => {
                return;
            }
        };

        match value ^ modifiers.negative {
            true => self.buffer.set(x, y),
            false => self.buffer.reset(x, y),
        }
    }

    fn translate(
        &self,
        x: isize,
        y: isize,
        area: &Rectangle,
        modifiers: &Modifiers,
    ) -> Option<(usize, usize)> {
        let (x, y) = match modifiers.flip_vertical {
            true => (x, area.size.height as isize - y - 1),
            false => (x, y),
        };

        let (x, y) = match modifiers.flip_horizontal {
            true => (area.size.width as isize - x - 1, y),
            false => (x, y),
        };

        if x < 0 || y < 0 {
            return None;
        }

        let (x, y) = (area.position.x + x as usize, area.position.y + y as usize);
        match x < self.buffer.width() && y < self.buffer.height() {
            true => Some((x, y)),
            false => None,
        }
    }
}

impl UserData for Buffer {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("bytes", |_lua, buffer, _: ()| Ok(buffer.bytes().to_vec()));
    }
}

pub trait BufferTrait {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn bytes(&self) -> &Vec<u8>;
    #[allow(unused)]
    fn get(&self, x: usize, y: usize) -> Option<bool>;
    fn set(&mut self, x: usize, y: usize);
    fn reset(&mut self, x: usize, y: usize);
}

pub struct ByteBuffer {
    width_px: usize,
    height_px: usize,
    data: Vec<u8>,
}

impl ByteBuffer {
    pub fn new(size: Size) -> Self {
        Self {
            width_px: size.width,
            height_px: size.height,
            data: vec![0; size.height * size.width],
        }
    }

    fn byte_position(&self, x: usize, y: usize) -> Option<usize> {
        if x >= self.width_px || y >= self.height_px {
            return None;
        }

        let index = y * self.width_px + x;
        Some(index)
    }

    fn get_byte(&self, x: usize, y: usize) -> Option<&u8> {
        self.byte_position(x, y)
            .and_then(|index| Some(&self.data[index]))
    }

    fn get_byte_mut(&mut self, x: usize, y: usize) -> Option<&mut u8> {
        self.byte_position(x, y)
            .and_then(|index| Some(&mut self.data[index]))
    }
}

impl BufferTrait for ByteBuffer {
    fn width(&self) -> usize {
        self.width_px
    }

    fn height(&self) -> usize {
        self.height_px
    }

    fn bytes(&self) -> &Vec<u8> {
        &self.data
    }

    fn get(&self, x: usize, y: usize) -> Option<bool> {
        self.get_byte(x, y).and_then(|value| Some(*value > 0))
    }

    fn set(&mut self, x: usize, y: usize) {
        if let Some(value) = self.get_byte_mut(x, y) {
            *value = 0xFF;
        }
    }

    fn reset(&mut self, x: usize, y: usize) {
        if let Some(value) = self.get_byte_mut(x, y) {
            *value = 0x00;
        }
    }
}

pub struct BitBuffer {
    width_px: usize,
    height_px: usize,
    width_bytes: usize,
    data: Vec<u8>,
}

impl BitBuffer {
    pub fn new(size: Size) -> Self {
        let width_bytes = (size.width + 7) / 8;
        Self {
            width_px: size.width,
            height_px: size.height,
            width_bytes,
            data: vec![0; size.height * width_bytes],
        }
    }

    fn bit_position(&self, x: usize, y: usize) -> Option<(usize, usize)> {
        if x >= self.width_px || y >= self.height_px {
            return None;
        }

        let index = y * self.width_bytes + x / 8;
        let offset = 7 - x % 8;

        Some((index, offset))
    }

    fn get_bit(&self, x: usize, y: usize) -> Option<Bit> {
        self.bit_position(x, y)
            .and_then(|(index, offset)| Some(Bit::new(&self.data[index], offset)))
    }

    fn get_bit_mut(&mut self, x: usize, y: usize) -> Option<BitMut> {
        self.bit_position(x, y)
            .and_then(|(index, offset)| Some(BitMut::new(&mut self.data[index], offset)))
    }
}

impl BufferTrait for BitBuffer {
    fn width(&self) -> usize {
        self.width_px
    }

    fn height(&self) -> usize {
        self.height_px
    }

    fn bytes(&self) -> &Vec<u8> {
        &self.data
    }

    fn get(&self, x: usize, y: usize) -> Option<bool> {
        self.get_bit(x, y).and_then(|bit| Some(bit.get()))
    }

    fn set(&mut self, x: usize, y: usize) {
        if let Some(mut bit) = self.get_bit_mut(x, y) {
            bit.set();
        }
    }

    fn reset(&mut self, x: usize, y: usize) {
        if let Some(mut bit) = self.get_bit_mut(x, y) {
            bit.reset();
        }
    }
}

pub struct VerticalBitBuffer {
    width_px: usize,
    height_px: usize,
    _height_bytes: usize,
    data: Vec<u8>,
}

impl VerticalBitBuffer {
    pub fn new(size: Size) -> Self {
        let height_bytes = (size.height + 7) / 8;
        Self {
            width_px: size.width,
            height_px: size.height,
            _height_bytes: height_bytes,
            data: vec![0; size.width * height_bytes],
        }
    }

    fn bit_position(&self, x: usize, y: usize) -> Option<(usize, usize)> {
        if x >= self.width_px || y >= self.height_px {
            return None;
        }

        let index = y / 8 * self.width_px + x;
        let offset = y % 8;

        Some((index, offset))
    }

    fn get_bit(&self, x: usize, y: usize) -> Option<Bit> {
        self.bit_position(x, y)
            .and_then(|(index, offset)| Some(Bit::new(&self.data[index], offset)))
    }

    fn get_bit_mut(&mut self, x: usize, y: usize) -> Option<BitMut> {
        self.bit_position(x, y)
            .and_then(|(index, offset)| Some(BitMut::new(&mut self.data[index], offset)))
    }
}

impl BufferTrait for VerticalBitBuffer {
    fn width(&self) -> usize {
        self.width_px
    }

    fn height(&self) -> usize {
        self.height_px
    }

    fn bytes(&self) -> &Vec<u8> {
        &self.data
    }

    fn get(&self, x: usize, y: usize) -> Option<bool> {
        self.get_bit(x, y).and_then(|bit| Some(bit.get()))
    }

    fn set(&mut self, x: usize, y: usize) {
        if let Some(mut bit) = self.get_bit_mut(x, y) {
            bit.set();
        }
    }

    fn reset(&mut self, x: usize, y: usize) {
        if let Some(mut bit) = self.get_bit_mut(x, y) {
            bit.reset();
        }
    }
}
