use crate::model::operation::Modifiers;
use crate::model::rectangle::{Rectangle, Size};

pub struct Buffer {
    height: usize,
    width: usize,
    buffer: Vec<u8>,
}

impl Buffer {
    pub fn new(size: Size) -> Self {
        assert_eq!(size.width % 8, 0);
        Self {
            width: size.width,
            height: size.height,
            buffer: vec![0; size.height * size.width / 8],
        }
    }

    pub fn fill(&mut self, area: &Rectangle, modifiers: &Modifiers) {
        if !modifiers.inverted && !modifiers.strict {
            // since buffer is empty by default we can save some time by skipping a lot of
            // potential no-ops but may leave some pixels set in other areas that didn't use the
            // strict mode or were overlapping
            return;
        }

        for y in 0..area.size.height {
            for x in 0..area.size.width {
                self.reset(y, x, area, modifiers)
            }
        }
    }

    pub fn set(&mut self, y: usize, x: usize, area: &Rectangle, modifiers: &Modifiers) {
        let (row, col) = match self.translate(y, x, area, modifiers) {
            Some(pos) => pos,
            None => { return; }
        };

        let func = match modifiers.inverted {
            true => |buf: &mut Vec<u8>, index: usize, mask: u8| { buf[index] &= !mask },
            false => |buf: &mut Vec<u8>, index: usize, mask: u8| { buf[index] |= mask }
        };

        let (index, mask) = self.get_index_and_mask(row, col);
        func(&mut self.buffer, index, mask)
    }

    pub fn reset(&mut self, y: usize, x: usize, area: &Rectangle, modifiers: &Modifiers) {
        // TODO extract common parts from set and reset
        let (row, col) = match self.translate(y, x, area, modifiers) {
            Some(pos) => pos,
            None => { return; }
        };

        let func = match modifiers.inverted {
            true => |buf: &mut Vec<u8>, index: usize, mask: u8| { buf[index] |= mask },
            false => |buf: &mut Vec<u8>, index: usize, mask: u8| { buf[index] &= !mask },
        };

        let (index, mask) = self.get_index_and_mask(row, col);
        func(&mut self.buffer, index, mask)
    }

    fn get_index_and_mask(&self, y: usize, x: usize) -> (usize, u8) {
        ((y * self.width + x) / 8, (1 as u8) << ((7 - x % 8) as u8))
    }

    fn translate(&self, y: usize, x: usize, area: &Rectangle, modifiers: &Modifiers) -> Option<(usize, usize)> {
        let (y, x) = match modifiers.flip_vertical {
            true => (area.size.height - y, x),
            false => (y, x)
        };

        let (y, x) = match modifiers.flip_horizontal {
            true => (y, area.size.width - x),
            false => (y, x)
        };

        let (y, x) = (y + area.origin.y, x + area.origin.x);
        match y < self.height && x < self.width {
            true => Some((y, x)),
            false => None
        }
    }
}

impl Into<Vec<u8>> for Buffer {
    fn into(self) -> Vec<u8> {
        self.buffer
    }
}