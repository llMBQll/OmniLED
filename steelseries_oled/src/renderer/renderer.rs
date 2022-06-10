use freetype as ft;

struct Position {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
}

struct Renderer {
    width: usize,
    height: usize,
}

impl Renderer {
    pub fn new(width: usize, height: usize) -> Self {
        assert_eq!(width % 8, 0);
        Self {
            width,
            height,
        }
    }

    pub fn render(&self) -> Vec<u8> {
        let mut buffer = vec![0 as u8; self.width * self.height / 8];

        buffer
    }

    fn render_single(buffer: &mut Vec<u8>, pos: Position) {

    }
}