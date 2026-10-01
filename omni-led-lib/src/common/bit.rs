pub struct Bit<'a> {
    byte: &'a u8,
    offset: usize,
}

impl<'a> Bit<'a> {
    pub fn new(byte: &'a u8, offset: usize) -> Self {
        Self { byte, offset }
    }

    pub fn get(&self) -> bool {
        get(*self.byte, self.offset)
    }
}

pub struct BitMut<'a> {
    byte: &'a mut u8,
    offset: usize,
}

impl<'a> BitMut<'a> {
    pub fn new(byte: &'a mut u8, offset: usize) -> Self {
        Self { byte, offset }
    }

    #[allow(unused)]
    pub fn get(&self) -> bool {
        get(*self.byte, self.offset)
    }

    pub fn set(&mut self) {
        set(self.byte, self.offset)
    }

    pub fn reset(&mut self) {
        reset(self.byte, self.offset)
    }
}

const MASK: [u8; 8] = [
    0b00000001, 0b00000010, 0b00000100, 0b00001000, 0b00010000, 0b00100000, 0b01000000, 0b10000000,
];

fn get(byte: u8, offset: usize) -> bool {
    (byte & MASK[offset]) != 0
}

fn set(byte: &mut u8, offset: usize) {
    *byte |= MASK[offset]
}

fn reset(byte: &mut u8, offset: usize) {
    *byte &= !MASK[offset]
}
