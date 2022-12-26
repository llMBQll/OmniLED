use std::cmp::max;
use crate::model::operation::{Operation, TextModifiers};
use crate::model::position::Position;
use crate::renderer::font_manager::FontManager;
use crate::renderer::screen::Screen;

pub struct Renderer {
    height: usize,
    width: usize,
    font_manager: FontManager,
    scrolling_text_data: ScrollingTextData,
}

impl Renderer {
    pub fn new(height: usize, width: usize) -> Self {
        assert_eq!(width % 8, 0);
        Self {
            height,
            width,
            font_manager: FontManager::new(),
            scrolling_text_data: ScrollingTextData::new(),
        }
    }

    pub fn render(&mut self, operations: Vec<Operation>) -> Vec<u8> {
        let mut screen = Screen::new(self.height, self.width);
        self.calculate_scrolling_text_data(&operations);
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
                self.render_text(screen, text.position, text.text, text.modifiers)
            }
            Operation::ScrollingText(text) => {
                self.render_scrolling_text(screen, text.position, text.text, text.modifiers, self.scrolling_text_data.ticks)
            }
        }
    }

    fn render_bar(&mut self, screen: &mut Screen, pos: Position, percent: f32) {
        let width = pos.width as f32 * percent / 100.0;

        for row in 0..pos.height {
            for col in 0..width as usize {
                screen.set(pos.y + row, pos.x + col);
            }
        }
    }

    fn get_text_height(height: usize, upper: bool) -> usize {
        if upper { height * 40 / 29 } else { height }
    }

    fn render_text(&mut self, screen: &mut Screen, pos: Position, text: String, modifiers: TextModifiers) {
        let mut cursor_x = 0 as i32;
        let cursor_y = pos.height as i32;

        let height = Self::get_text_height(pos.height, modifiers.upper);
        for character in text.chars() {
            let character = self.font_manager.get_character(character as usize, height);
            let bitmap = &character.bitmap;
            let metrics = &character.metrics;

            for row in 0..bitmap.rows {
                for col in 0..bitmap.cols {
                    let offset_y = (metrics.horiBearingY >> 6) as i32;
                    let y = cursor_y + row as i32 - offset_y;
                    let x = cursor_x + col as i32 + (metrics.horiBearingX >> 6) as i32;

                    if y < 0 || x < 0 || x >= pos.width as i32 || (modifiers.strict && y >= pos.height as i32) {
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

    fn calculate_scrolling_text_data(&mut self, operations: &Vec<Operation>) {
        let mut vec = Vec::new();
        for op in operations {
            match op {
                Operation::ScrollingText(text) => {
                    let res = self.calculate_scrolling_text(&text.position, &text.text, &text.modifiers, Some(text.count));
                    vec.push(res);
                }
                _ => {}
            }
        }
        if !vec.is_empty() {
            vec.sort_by(|(a, _), (b, _)| b.cmp(a));
            self.scrolling_text_data.ticks = vec[0].1;
        }
    }

    fn calculate_scrolling_text(&mut self, pos: &Position, text: &String, modifiers: &TextModifiers, count: Option<i32>) -> (usize, usize) {
        // count is required to calculate tick, so if it is already known then it can be omitted

        let height = Self::get_text_height(pos.height, modifiers.upper);
        let width = pos.width;
        let character = self.font_manager.get_character('a' as usize, height);
        let char_width = (character.metrics.horiAdvance >> 6) as usize;
        let max_chars = width / max(char_width, 1);
        let len = text.len();

        if len <= max_chars {
            return (0, 0);
        }

        let shifts = len - max_chars;
        let max_ticks = 2 * TICKS_AT_EDGE + shifts * TICKS_PER_MOVE;
        let tick = count.unwrap_or(0) as usize % max_ticks;

        (shifts, tick)
    }

    fn render_scrolling_text(&mut self, screen: &mut Screen, pos: Position, text: String, modifiers: TextModifiers, tick: usize) {
        // Don't care about tick as we use the tick of a value that will have to be shifted the most times
        // This way all scrolling texts will be synchronized and will reset after last shift of the item that
        // requires the most shifts

        let (shifts, _tick) = self.calculate_scrolling_text(&pos, &text, &modifiers, None);

        if shifts == 0 {
            self.render_text(screen, pos, text, modifiers)
        } else {
            let offset = if tick <= TICKS_AT_EDGE {
                0
            } else if tick >= TICKS_AT_EDGE + shifts * TICKS_PER_MOVE {
                shifts
            } else {
                (tick - TICKS_AT_EDGE) / TICKS_PER_MOVE
            };

            let mut chars = text.chars();
            for _ in 0..offset {
                let _ = chars.next();
            }
            let substr: String = chars.collect();

            self.render_text(screen, pos, substr, modifiers)
        }
    }
}

const TICKS_PER_MOVE: usize = 2;
const TICKS_AT_EDGE: usize = 8;

unsafe impl Send for Renderer {}

struct ScrollingTextData {
    pub ticks: usize,
}

impl ScrollingTextData {
    pub fn new() -> Self {
        Self {
            ticks: 0,
        }
    }
}