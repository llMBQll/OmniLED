use crate::model::operation::Operation;
use crate::Position;
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

    pub fn render(&mut self, operations: Vec<Operation>) -> Vec<u8> {
        let mut screen = Screen::new(self.height, self.width);
        for operation in operations {
            self.perform_operation(&mut screen, operation);
        }
        screen.into()
    }

    fn perform_operation(&mut self, screen: &mut Screen, op: Operation) {
        match op {
            Operation::Bar(bar) => {
                self.render_bar(screen, bar.position, bar.value)
            }
            Operation::Text(text) => {
                self.render_text(screen, text.position, text.text)
            },
            Operation::FixedHeight(fixed_height) => {
                self.render_text(screen, fixed_height.position, fixed_height.text)
            }
            Operation::ScrollingText(_scrolling_text) => {
                todo!()
            }
        }
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
                    if bitmap[(row, col)] > 38 {
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

    fn render_bar(&mut self, screen: &mut Screen, pos: Position, percent: f32) {
        let width = pos.width as f32 * percent / 100.0;

        for row in 0..pos.height {
            for col in 0..width as usize  {
                screen.set(pos.y + row, pos.x + col);
            }
        }
    }
}