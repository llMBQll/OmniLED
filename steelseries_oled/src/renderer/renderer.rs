use mlua::Lua;
use std::cmp::max;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::vec::IntoIter;

use crate::common::user_data::UserDataRef;
use crate::renderer::buffer::{BitBuffer, Buffer, ByteBuffer};
use crate::renderer::font_manager::FontManager;
use crate::script_handler::script_data_types::{
    MemoryRepresentation, Modifiers, OledImage, Operation, Point, Text,
};
use crate::script_handler::script_data_types::{Rectangle, Size};
use crate::settings::settings::Settings;

pub struct Renderer {
    font_manager: FontManager,
    scrolling_text_data: ScrollingTextData,
    scrolling_text_control: ScrollingTextControl,
}

impl Renderer {
    pub fn new(lua: &Lua) -> Self {
        let settings = UserDataRef::<Settings>::load(lua);
        let font_selector = settings.get().font.clone();

        Self {
            font_manager: FontManager::new(font_selector),
            scrolling_text_data: ScrollingTextData::new(),
            scrolling_text_control: ScrollingTextControl::new(lua),
        }
    }

    pub fn render(
        &mut self,
        ctx: ContextKey,
        size: Size,
        operations: Vec<Operation>,
        memory_representation: MemoryRepresentation,
    ) -> (bool, Buffer) {
        let mut buffer = match memory_representation {
            MemoryRepresentation::BitPerPixel => Buffer::new(BitBuffer::new(size)),
            MemoryRepresentation::BytePerPixel => Buffer::new(ByteBuffer::new(size)),
        };
        let (end_auto_repeat, text_offsets) = self.precalculate_text(ctx, &operations);
        let mut text_offsets = text_offsets.into_iter();

        for operation in operations {
            match operation {
                Operation::Bar(bar) => Self::render_bar(
                    &mut buffer,
                    bar.position,
                    bar.size,
                    bar.value,
                    bar.modifiers,
                ),
                Operation::Image(image) => Self::render_image(
                    &mut buffer,
                    image.position,
                    image.size,
                    image.image,
                    image.modifiers,
                ),
                Operation::Text(text) => self.render_text(
                    &mut buffer,
                    text.position,
                    text.size,
                    text.text,
                    text.modifiers,
                    &mut text_offsets,
                ),
            }
        }

        (end_auto_repeat, buffer)
    }

    fn render_bar(
        buffer: &mut Buffer,
        position: Point,
        size: Size,
        value: f32,
        modifiers: Modifiers,
    ) {
        let (height, width) = match modifiers.vertical {
            true => ((size.height as f32 * value / 100.0) as usize, size.width),
            false => (size.height, (size.width as f32 * value / 100.0) as usize),
        };

        let rect = Rectangle { position, size };
        for y in 0..height as isize {
            for x in 0..width as isize {
                buffer.set(x, y, &rect, &modifiers);
            }
        }
    }

    fn render_image(
        buffer: &mut Buffer,
        position: Point,
        size: Size,
        image: OledImage,
        modifiers: Modifiers,
    ) {
        if size.width == 0 || size.height == 0 {
            return;
        }

        let x_factor = image.size.width as f64 / size.width as f64;
        let y_factor = image.size.height as f64 / size.height as f64;

        let rect = Rectangle { position, size };
        for y in 0..size.height as isize {
            for x in 0..size.width as isize {
                // Use nearest neighbour interpolation for now as it's the quickest to implement
                // TODO allow specifying scaling algorithm as modifier
                // TODO potentially cache scaled images

                let image_x = (x as f64 * x_factor).round() as usize;
                let image_x = image_x.clamp(0, image.size.width - 1);

                let image_y = (y as f64 * y_factor).round() as usize;
                let image_y = image_y.clamp(0, image.size.height - 1);

                let index = image_y * image.size.width + image_x;
                if image.bytes[index] != 0 {
                    buffer.set(x, y, &rect, &modifiers);
                }
            }
        }
    }

    fn render_text(
        &mut self,
        buffer: &mut Buffer,
        position: Point,
        size: Size,
        text: String,
        modifiers: Modifiers,
        offsets: &mut IntoIter<usize>,
    ) {
        let mut cursor_x = 0;
        let cursor_y = size.height as isize;

        let offset = offsets.next().expect("Each 'Text' shall have its offset");
        let mut characters = text.chars();
        for _ in 0..offset {
            _ = characters.next();
        }

        let rect = Rectangle { position, size };
        let character_height = modifiers.font_size.unwrap_or(rect.size.height);
        for character in characters {
            let character = self.font_manager.get_character(character, character_height);
            let bitmap = &character.bitmap;
            let metrics = &character.metrics;

            for bitmap_y in 0..bitmap.rows as isize {
                for bitmap_x in 0..bitmap.cols as isize {
                    let x = cursor_x + bitmap_x + metrics.offset_x;
                    let y = cursor_y + bitmap_y - metrics.offset_y;

                    if x < 0
                        || y < 0
                        || (modifiers.strict && x >= rect.size.width as isize)
                        || (modifiers.strict && y >= rect.size.height as isize)
                    {
                        continue;
                    }

                    if bitmap.get(bitmap_x as usize, bitmap_y as usize) {
                        buffer.set(x, y, &rect, &modifiers);
                    }
                }
            }

            cursor_x += metrics.advance;
            if cursor_x > rect.size.width as isize {
                break;
            }
        }
    }

    fn precalculate_text(
        &mut self,
        ctx: ContextKey,
        operations: &Vec<Operation>,
    ) -> (bool, Vec<usize>) {
        let mut ctx = self.scrolling_text_data.get_context(ctx);

        let offsets: Vec<usize> = operations
            .iter()
            .filter_map(|op| match op {
                Operation::Text(text) => Some(Self::precalculate_single(
                    &mut ctx,
                    &mut self.font_manager,
                    &self.scrolling_text_control,
                    text,
                )),
                _ => None,
            })
            .collect();

        match ctx.was_reset() {
            true => (false, vec![0; offsets.len()]),
            false => (ctx.can_wrap(), offsets),
        }
    }

    fn precalculate_single(
        ctx: &mut Context,
        font_manager: &mut FontManager,
        control: &ScrollingTextControl,
        text: &Text,
    ) -> usize {
        if !text.modifiers.scrolling {
            return 0;
        }

        let height = text.modifiers.font_size.unwrap_or(text.size.height);
        let width = text.size.width;
        let character = font_manager.get_character('a', height);
        let char_width = character.metrics.advance as usize;
        let max_characters = width / max(char_width, 1);
        let len = text.text.chars().count();
        let tick = ctx.read(&text.text);

        if len <= max_characters {
            ctx.set(&text.text, true);
            return 0;
        }

        let max_shifts = len - max_characters;
        let max_ticks = 2 * control.ticks_at_edge + max_shifts * control.ticks_per_move;
        if tick >= max_ticks {
            ctx.set(&text.text, true);
        }

        if tick <= control.ticks_at_edge {
            0
        } else if tick >= control.ticks_at_edge + max_shifts * control.ticks_per_move {
            max_shifts
        } else {
            (tick - control.ticks_at_edge) / control.ticks_per_move
        }
    }
}

struct ScrollingTextControl {
    ticks_at_edge: usize,
    ticks_per_move: usize,
}

impl ScrollingTextControl {
    pub fn new(lua: &Lua) -> Self {
        let settings = UserDataRef::<Settings>::load(lua);
        let text_control = Self {
            ticks_at_edge: settings.get().text_ticks_scroll_delay,
            ticks_per_move: settings.get().text_ticks_scroll_rate,
        };
        text_control
    }
}

struct ScrollingTextData {
    contexts: HashMap<ContextKey, HashMap<String, usize>>,
}

impl ScrollingTextData {
    pub fn new() -> Self {
        Self {
            contexts: HashMap::new(),
        }
    }

    pub fn get_context(&mut self, ctx: ContextKey) -> Context {
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
        // TODO what is happening here
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
        self.context_info.iter().all(|(_, can_wrap)| *can_wrap)
    }
}

impl<'a> Drop for Context<'a> {
    fn drop(&mut self) {
        if self.reset {
            // remove all stale entries from map - old keys will not be present in context_info
            *self.map = self
                .map
                .iter()
                .filter_map(|(key, value)| match self.context_info.contains_key(key) {
                    true => Some((key.clone(), *value)),
                    false => None,
                })
                .collect();
        }

        if self.reset || self.can_wrap() {
            for (_, tick) in &mut *self.map {
                *tick = 0;
            }
        }
    }
}

#[derive(Eq, Hash, PartialEq)]
pub struct ContextKey {
    pub script: usize,
    pub device: usize,
}
