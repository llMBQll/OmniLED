use crate::model::operation::Modifiers;
use crate::model::rectangle::{Rectangle, Size};
use crate::renderer::bit::Bit;

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

    pub fn set(&mut self, y: usize, x: usize, area: &Rectangle, modifiers: &Modifiers) {
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
        let index = (y * self.width + x) / 8;
        Bit::new(&mut self.buffer[index], 7 - x % 8)
    }

    fn translate(
        &self,
        y: usize,
        x: usize,
        area: &Rectangle,
        modifiers: &Modifiers,
    ) -> Option<(usize, usize)> {
        let (y, x) = match modifiers.flip_vertical {
            true => (area.size.height - y, x),
            false => (y, x),
        };

        let (y, x) = match modifiers.flip_horizontal {
            true => (y, area.size.width - x),
            false => (y, x),
        };

        let (y, x) = (y + area.origin.y, x + area.origin.x);
        match y < self.height && x < self.width {
            true => Some((y, x)),
            false => None,
        }
    }
}

impl Into<Vec<u8>> for Buffer {
    fn into(self) -> Vec<u8> {
        self.buffer
    }
}
