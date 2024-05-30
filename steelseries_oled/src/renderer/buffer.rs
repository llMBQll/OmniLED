use crate::renderer::bit::Bit;
use crate::script_handler::script_data_types::Modifiers;
use crate::script_handler::script_data_types::{Rectangle, Size};

pub struct Buffer {
    width: usize,
    height: usize,
    padded_width: usize,
    buffer: Vec<u8>,
}

impl Buffer {
    pub fn new(size: Size) -> Self {
        let oversize = size.width % 8;
        let padding = if oversize == 0 { 0 } else { 8 - oversize };
        let padded_width = size.width + padding;
        Self {
            width: size.width,
            height: size.height,
            padded_width,
            buffer: vec![0; size.height * padded_width / 8],
        }
    }

    pub fn set(&mut self, x: isize, y: isize, area: &Rectangle, modifiers: &Modifiers) {
        let (x, y) = match self.translate(x, y, area, modifiers) {
            Some(pos) => pos,
            None => {
                return;
            }
        };

        let mut bit = self.bit_at(x, y);
        match modifiers.strict || !bit.get() {
            true => bit.set(),
            false => bit.reset(),
        };
    }

    fn bit_at(&mut self, x: usize, y: usize) -> Bit {
        let index = (y * self.padded_width + x) / 8;
        Bit::new(&mut self.buffer[index], 7 - x % 8)
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

        let (x, y) = (x + area.origin.x as isize, y + area.origin.y as isize);
        match x >= 0 && x < self.width as isize && y >= 0 && y < self.height as isize {
            true => Some((x as usize, y as usize)),
            false => None,
        }
    }
}

impl Into<Vec<u8>> for Buffer {
    fn into(self) -> Vec<u8> {
        self.buffer
    }
}
