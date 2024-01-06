use crate::model::operation::Modifiers;
use crate::model::rectangle::{Rectangle, Size};
use crate::renderer::bit::Bit;

pub struct Buffer {
    height: usize,
    width: usize,
    padded_width: usize,
    buffer: Vec<u8>,
}

impl Buffer {
    pub fn new(size: Size) -> Self {
        let oversize = size.width % 8;
        let padding = if oversize == 0 { 0 } else { 8 - oversize };
        let padded_width = size.width + padding;
        Self {
            height: size.height,
            width: size.width,
            padded_width,
            buffer: vec![0; size.height * padded_width / 8],
        }
    }

    pub fn set(&mut self, y: isize, x: isize, area: &Rectangle, modifiers: &Modifiers) {
        let (row, col) = match self.translate(y, x, area, modifiers) {
            Some(pos) => pos,
            None => {
                return;
            }
        };

        let mut bit = self.bit_at(row, col);
        match modifiers.strict || !bit.get() {
            true => bit.set(),
            false => bit.reset(),
        };
    }

    fn bit_at(&mut self, y: usize, x: usize) -> Bit {
        let index = (y * self.padded_width + x) / 8;
        Bit::new(&mut self.buffer[index], 7 - x % 8)
    }

    fn translate(
        &self,
        y: isize,
        x: isize,
        area: &Rectangle,
        modifiers: &Modifiers,
    ) -> Option<(usize, usize)> {
        let (y, x) = match modifiers.flip_vertical {
            true => (area.size.height as isize - y, x),
            false => (y, x),
        };

        let (y, x) = match modifiers.flip_horizontal {
            true => (y, area.size.width as isize - x),
            false => (y, x),
        };

        let (y, x) = (y + area.origin.y as isize, x + area.origin.x as isize);
        match y >= 0 && y < self.height as isize && x >= 0 && x < self.width as isize {
            true => Some((y as usize, x as usize)),
            false => None,
        }
    }
}

impl Into<Vec<u8>> for Buffer {
    fn into(self) -> Vec<u8> {
        self.buffer
    }
}
