pub struct Screen {
    height: usize,
    width: usize,
    buffer: Vec<u8>,
}

impl Screen {
    pub fn new(height: usize, width: usize) -> Self {
        assert_eq!(width % 8, 0);
        Self {
            width,
            height,
            buffer: vec![0; height * width / 8]
        }
    }

    #[allow(unused)]
    pub fn get(&self, row: usize, col: usize) -> bool {
        let (index, mask) = self.get_index_and_mask(row, col);
        self.buffer[index] & mask != 0
    }

    pub fn set(&mut self, row: usize, col: usize) {
        if row >= self.height || col >= self.width {
            return;
        }
        let (index, mask) = self.get_index_and_mask(row, col);
        self.buffer[index] |= mask;
    }

    #[allow(unused)]
    pub fn reset(&mut self, row: usize, col: usize) {
        let (index, mask) = self.get_index_and_mask(row, col);
        self.buffer[index] &= !mask
    }

    fn get_index_and_mask(&self, row: usize, col: usize) -> (usize, u8) {
        ((row * self.width + col) / 8, (1 as u8) << ((7 - col % 8) as u8))
    }
}

impl Into<Vec<u8>> for Screen {
    fn into(self) -> Vec<u8> {
        self.buffer
    }
}