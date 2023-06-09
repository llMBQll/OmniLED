use std::cmp::max;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::vec::IntoIter;
use mlua::{Lua, UserData, UserDataMethods};
use crate::model::operation::{Operation, Modifiers, Text};
use crate::model::rectangle::{Rectangle, Size};
use crate::renderer::font_manager::FontManager;
use crate::renderer::buffer::Buffer;

pub struct Renderer {
    font_manager: FontManager,
    scrolling_text_data: ScrollingTextData,
}

const TICKS_PER_MOVE: usize = 2;
const TICKS_AT_EDGE: usize = 8;

impl Renderer {
    pub fn load(lua: &Lua) {
        let renderer = Self::new();
        lua.globals().set("RENDERER", renderer).unwrap();
    }

    pub fn new() -> Self {
        Self {
            font_manager: FontManager::new(),
            scrolling_text_data: ScrollingTextData::new(),
        }
    }

    pub fn render(&mut self, ctx: i64, size: Size, operations: Vec<Operation>) -> (bool, Vec<u8>) {
        let mut buffer = Buffer::new(size);
        let (end_auto_repeat, text_offsets) = self.precalculate_text(ctx, &operations);
        let mut text_offsets = text_offsets.into_iter();

        for operation in operations {
            match operation {
                Operation::Bar(bar) => {
                    self.render_bar(&mut buffer, bar.position, bar.value, bar.modifiers)
                }
                Operation::Text(text) => {
                    self.render_text(&mut buffer, text.position, text.text, text.modifiers, &mut text_offsets)
                }
            }
        }

        (end_auto_repeat, buffer.into())
    }

    fn render_bar(&self, buffer: &mut Buffer, rect: Rectangle, value: f32, modifiers: Modifiers) {
        // TODO: consider making it a static function
        let (height, width) = match modifiers.vertical {
            true => ((rect.size.height as f32 * value / 100.0) as usize, rect.size.width),
            false => (rect.size.height, (rect.size.width as f32 * value / 100.0) as usize)
        };

        for row in 0..height {
            for col in 0..width {
                buffer.set(row, col, &rect, &modifiers);
            }
        }
    }

    fn get_text_height(height: usize, upper: bool) -> usize {
        if upper { height * 40 / 29 } else { height }
    }

    fn render_text(&mut self, buffer: &mut Buffer, rect: Rectangle, text: String, modifiers: Modifiers, offsets: &mut IntoIter<usize>) {
        const RENDER_THRESHOLD: u8 = 50;

        let mut cursor_x = 0 as i32;
        let cursor_y = rect.size.height as i32;

        let offset = offsets.next().unwrap();
        let mut characters = text.chars();
        for _ in 0..offset {
            _ = characters.next();
        }

        let character_height = Self::get_text_height(rect.size.height, modifiers.upper);
        for character in characters {
            let character = self.font_manager.get_character(character as usize, character_height);
            let bitmap = &character.bitmap;
            let metrics = &character.metrics;

            for row in 0..bitmap.rows {
                for col in 0..bitmap.cols {
                    let offset_y = (metrics.horiBearingY >> 6) as i32;
                    let y = cursor_y + row as i32 - offset_y;
                    let x = cursor_x + col as i32 + (metrics.horiBearingX >> 6) as i32;

                    if y < 0 || x < 0 || x >= rect.size.width as i32 || (modifiers.strict && y >= rect.size.height as i32) {
                        continue;
                    }
                    if bitmap[(row, col)] > RENDER_THRESHOLD {
                        buffer.set(y as usize, x as usize, &rect, &modifiers);
                    }
                }
            }

            cursor_x += (metrics.horiAdvance >> 6) as i32;
            if cursor_x > rect.size.width as i32 {
                break;
            }
        }
    }

    fn precalculate_text(&mut self, ctx: i64, operations: &Vec<Operation>) -> (bool, Vec<usize>) {
        let mut ctx = self.scrolling_text_data.get_context(ctx);

        let offsets: Vec<usize> = operations.iter().filter_map(|op| {
            match op {
                Operation::Text(text) => Some(Self::precalculate_single(&mut ctx, &mut self.font_manager, text)),
                _ => None
            }
        }).collect();

        match ctx.was_reset() {
            true => (false, vec![0; offsets.len()]),
            false => (ctx.can_wrap(), offsets)
        }
    }

    fn precalculate_single(ctx: &mut Context, font_manager: &mut FontManager, text: &Text) -> usize {
        if !text.modifiers.scrolling {
            return 0;
        }

        let height = Self::get_text_height(text.position.size.height, text.modifiers.upper);
        let width = text.position.size.width;
        let character = font_manager.get_character('a' as usize, height);
        let char_width = (character.metrics.horiAdvance >> 6) as usize;
        let max_characters = width / max(char_width, 1);
        let len = text.text.len();
        let tick = ctx.read(&text.text);

        if len <= max_characters {
            ctx.set(&text.text, true);
            return 0;
        }

        let max_shifts = len - max_characters;
        let max_ticks = 2 * TICKS_AT_EDGE + max_shifts * TICKS_PER_MOVE;
        if tick >= max_ticks {
            ctx.set(&text.text, true);
        }

        if tick <= TICKS_AT_EDGE {
            0
        } else if tick >= TICKS_AT_EDGE + max_shifts * TICKS_PER_MOVE {
            max_shifts
        } else {
            (tick - TICKS_AT_EDGE) / TICKS_PER_MOVE
        }
    }
}

impl UserData for Renderer {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("render", |_, this, (ctx, size, operations): (i64, Size, Vec<Operation>)| {
            Ok(this.render(ctx, size, operations))
        })
    }
}

unsafe impl Send for Renderer {}

struct ScrollingTextData {
    contexts: HashMap<i64, HashMap<String, usize>>,
}

impl ScrollingTextData {
    pub fn new() -> Self {
        Self {
            contexts: HashMap::new(),
        }
    }

    pub fn get_context(&mut self, ctx: i64) -> Context {
        let map = self.contexts.entry(ctx).or_insert(HashMap::new());
        Context::new(map)
    }
}

struct Context<'a> {
    map: &'a mut HashMap<String, usize>,
    context_info: HashMap<String, bool>,
    reset: bool,
}

impl<'a> Context<'a> {
    pub fn new(map: &'a mut HashMap<String, usize>) -> Self {
        Self {
            map,
            context_info: HashMap::new(),
            reset: false,
        }
    }

    pub fn read(&mut self, key: &String) -> usize {
        match self.context_info.entry(key.clone()) {
            Entry::Occupied(_) => {
                return *self.map.get(key).unwrap();
            }
            Entry::Vacant(entry) => {
                entry.insert(false);
            }
        }

        match self.map.entry(key.clone()) {
            Entry::Occupied(entry) => {
                let ticks = entry.get() + 1;
                *entry.into_mut() = ticks;
                ticks
            }
            Entry::Vacant(entry) => {
                self.reset = true;

                let ticks = 0;
                entry.insert(ticks);
                ticks
            }
        }
    }

    pub fn set(&mut self, key: &String, can_wrap: bool) {
        self.context_info.insert(key.clone(), can_wrap);
    }

    pub fn was_reset(&self) -> bool {
        self.reset
    }

    pub fn can_wrap(&self) -> bool {
        self.context_info.iter().all(|(_, can_wrap)| { *can_wrap })
    }
}

impl<'a> Drop for Context<'a> {
    fn drop(&mut self) {
        if self.reset {
            // remove all stale entries from map - old keys will not be present in context_info
            *self.map = self.map.iter().filter_map(|(key, value)| match self.context_info.contains_key(key) {
                true => Some((key.clone(), *value)),
                false => None
            }).collect();
        }

        if self.reset || self.can_wrap() {
            for (_, tick) in &mut *self.map {
                *tick = 0;
            }
        }
    }
}