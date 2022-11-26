use crate::model::operation::Operation;
use crate::model::position::Position;
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
                self.render_text(screen, text.position, text.text, text.strict, text.upper)
            },
            Operation::ScrollingText(_scrolling_text) => {
                todo!()
            }
        }
    }

    fn render_text(&mut self, screen: &mut Screen, pos: Position, text: String, strict: bool, upper: bool) {
        let mut cursor_x = 0 as i32;
        let cursor_y = pos.height as i32;

        let height = if upper { pos.height * 40 / 29 } else { pos.height };
        for character in text.chars() {
            let character = self.font_manager.get_character(character as usize, height);
            let bitmap = &character.bitmap;
            let metrics = &character.metrics;

            for row in 0..bitmap.rows {
                for col in 0..bitmap.cols {
                    let offset_y = (metrics.horiBearingY >> 6) as i32;
                    let y = cursor_y + row as i32 - offset_y;
                    let x = cursor_x + col as i32 + (metrics.horiBearingX >> 6) as i32;

                    if y < 0 || x < 0 || x >= pos.width as i32 || (strict && y >= pos.height as i32) {
                        continue;
                    }
                    if bitmap[(row, col)] > 50 {
                        screen.set(y as usize + pos.y, x as usize + pos.x);
                    }
                }
            }

            cursor_x += (metrics.horiAdvance >> 6) as i32;
            if cursor_x > pos.width as i32 {
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