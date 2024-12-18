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

pub struct Bit<'a> {
    byte: &'a mut u8,
    offset: usize,
}

const MASK: [u8; 8] = [
    0b00000001, 0b00000010, 0b00000100, 0b00001000, 0b00010000, 0b00100000, 0b01000000, 0b10000000,
];

impl<'a> Bit<'a> {
    pub fn new(byte: &'a mut u8, offset: usize) -> Self {
        Self { byte, offset }
    }

    pub fn get(&self) -> bool {
        (*self.byte & MASK[self.offset]) != 0
    }

    pub fn set(&mut self) {
        *self.byte |= MASK[self.offset]
    }

    pub fn reset(&mut self) {
        *self.byte &= !MASK[self.offset]
    }
}
