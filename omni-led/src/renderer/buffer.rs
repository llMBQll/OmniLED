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

use crate::renderer::bit::Bit;
use crate::script_handler::script_data_types::Modifiers;
use crate::script_handler::script_data_types::{Rectangle, Size};

pub struct Buffer {
    buffer: Box<dyn BufferTrait>,
}

impl Buffer {
    pub fn new<T: BufferTrait + 'static>(buffer: T) -> Self {
        Self {
            buffer: Box::new(buffer),
        }
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

    pub fn rows(&self) -> Vec<&[u8]> {
        let chunk_size = self.buffer.row_stride();
        self.buffer.bytes().chunks(chunk_size).collect()
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
            true => (x, area.size.height as isize - y),
            false => (x, y),
        };

        let (x, y) = match modifiers.flip_horizontal {
            true => (area.size.width as isize - x, y),
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

        methods.add_method("rows", |_lua, buffer, _: ()| {
            Ok(buffer
                .rows()
                .into_iter()
                .map(|row| row.to_vec())
                .collect::<Vec<_>>())
        });
    }
}

pub trait BufferTrait {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn bytes(&self) -> &Vec<u8>;
    fn row_stride(&self) -> usize;
    #[allow(unused)]
    fn get(&mut self, x: usize, y: usize) -> Option<bool>;
    fn set(&mut self, x: usize, y: usize);
    fn reset(&mut self, x: usize, y: usize);
}

pub struct ByteBuffer {
    width: usize,
    height: usize,
    data: Vec<u8>,
}

impl ByteBuffer {
    pub fn new(size: Size) -> Self {
        Self {
            width: size.width,
            height: size.height,
            data: vec![0; size.height * size.width],
        }
    }

    fn bit_at(&mut self, x: usize, y: usize) -> Option<&mut u8> {
        if x >= self.width || y >= self.height {
            return None;
        }

        let index = y * self.width + x;
        Some(&mut self.data[index])
    }
}

impl BufferTrait for ByteBuffer {
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn bytes(&self) -> &Vec<u8> {
        &self.data
    }

    fn row_stride(&self) -> usize {
        self.width
    }

    fn get(&mut self, x: usize, y: usize) -> Option<bool> {
        self.bit_at(x, y).and_then(|value| Some(*value > 0))
    }

    fn set(&mut self, x: usize, y: usize) {
        if let Some(value) = self.bit_at(x, y) {
            *value = 0xFF;
        }
    }

    fn reset(&mut self, x: usize, y: usize) {
        if let Some(value) = self.bit_at(x, y) {
            *value = 0x00;
        }
    }
}

pub struct BitBuffer {
    width: usize,
    height: usize,
    padded_width: usize,
    data: Vec<u8>,
}

impl BitBuffer {
    pub fn new(size: Size) -> Self {
        let oversize = size.width % 8;
        let padding = if oversize == 0 { 0 } else { 8 - oversize };
        let padded_width = size.width + padding;
        Self {
            width: size.width,
            height: size.height,
            padded_width,
            data: vec![0; size.height * padded_width / 8],
        }
    }

    fn bit_at(&mut self, x: usize, y: usize) -> Option<Bit> {
        if x >= self.width || y >= self.height {
            return None;
        }

        let index = (y * self.padded_width + x) / 8;
        Some(Bit::new(&mut self.data[index], 7 - x % 8))
    }
}

impl BufferTrait for BitBuffer {
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn bytes(&self) -> &Vec<u8> {
        &self.data
    }
    fn row_stride(&self) -> usize {
        self.padded_width / 8
    }

    fn get(&mut self, x: usize, y: usize) -> Option<bool> {
        self.bit_at(x, y).and_then(|bit| Some(bit.get()))
    }

    fn set(&mut self, x: usize, y: usize) {
        if let Some(mut bit) = self.bit_at(x, y) {
            bit.set();
        }
    }

    fn reset(&mut self, x: usize, y: usize) {
        if let Some(mut bit) = self.bit_at(x, y) {
            bit.reset();
        }
    }
}
