use crate::renderer::font_manager::FontManager;
use crate::renderer::screen::Screen;

pub struct Renderer {
    height: usize,
    width: usize,
    font_manager: FontManager
}

impl Renderer {
    pub fn new(height: usize, width: usize) -> Self {
        assert_eq!(width % 8, 0);
        Self {
            height,
            width,
            font_manager: FontManager::new()
        }
    }

    pub fn render(&mut self, percent: u32) -> Vec<u8> {
        let mut screen = Screen::new(self.height, self.width);
        self.render_text(&mut screen, Position::new(0, 0, 11, self.width / 5), format!("{}%", percent));
        self.render_progress(&mut screen, Position::new(2, self.width / 5, 9, self.width * 5 / 6), percent as usize);
        self.render_text(&mut screen, Position::new(11, 0, 11, self.width / 2), String::from("XDDDDD"));
        self.render_text(&mut screen, Position::new(22, 0, 11, self.width), String::from("Omegalul"));
        self.render_progress(&mut screen, Position::new(38, 0, 2, self.width), percent as usize);
        screen.into()
    }

    fn render_text(&mut self, screen: &mut Screen, pos: Position, text: String) {
        let mut cursor_x = 0;

        for character in text.chars() {
            let character = self.font_manager.get_character(character as usize, pos.height);
            let bitmap = &character.bitmap;
            let top = bitmap.top;
            let left = bitmap.left;
            for row in 0..bitmap.rows {
                for col in 0..bitmap.cols {
                    let y = pos.height as i32 + row as i32 - top;
                    let x = cursor_x as i32 + col as i32 + left;
                    // if y < 0 || y >= pos.height as i32 || x < 0 || x >= pos.width as i32 {
                    //     continue;
                    // }
                    if y < 0 || x < 0 {
                        continue;
                    }
                    if bitmap[(row, col)] > 32 {
                        screen.set(y as usize + pos.y, x as usize + pos.x);
                    }
                }
            }

            let advance = character.bounding_box.x_max / 64 + 1;
            cursor_x += if advance > 0 { advance as usize } else { 0 };
            if cursor_x > pos.width {
                break;
            }
        }
    }

    fn render_progress(&mut self, screen: &mut Screen, pos: Position, percent: usize) {
        for row in 0..pos.height {
            for col in 0..pos.width * percent / 100  {
                screen.set(pos.y + row, pos.x + col);
            }
        }
    }
}


#[derive(Copy, Clone)]
struct Position {
    pub y: usize,
    pub x: usize,
    pub height: usize,
    pub width: usize,
}

impl Position {
    fn new(y: usize, x: usize, height: usize, width: usize) -> Self {
        Self {
            y,
            x,
            height,
            width,
        }
    }
}