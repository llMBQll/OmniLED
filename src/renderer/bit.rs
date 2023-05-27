pub struct Bit<'a> {
    byte: &'a mut u8,
    offset: usize,
}

const MASK: [u8; 8] = [
    0b00000001,
    0b00000010,
    0b00000100,
    0b00001000,
    0b00010000,
    0b00100000,
    0b01000000,
    0b10000000,
];

impl<'a> Bit<'a> {
    pub fn new(byte: &'a mut u8, offset: usize) -> Self {
        Self {
            byte,
            offset,
        }
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