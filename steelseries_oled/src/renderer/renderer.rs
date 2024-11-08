use mlua::Lua;
use num_traits::clamp;
use std::cmp::max;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::vec::IntoIter;

use crate::common::user_data::UserDataRef;
use crate::renderer::buffer::{BitBuffer, Buffer, ByteBuffer};
use crate::renderer::font_manager::FontManager;
use crate::script_handler::script_data_types::{
    MemoryRepresentation, Modifiers, Offset, OledImage, Operation, Point, Range, Text,
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
                    bar.vertical,
                    bar.range,
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
                    text.text_offset,
                    text.font_size,
                    text.modifiers,
                    &mut text_offsets,
                ),
            }
        }

        (end_auto_repeat, buffer)
    }

    fn clear_background(buffer: &mut Buffer, position: Point, size: Size, modifiers: &Modifiers) {
        let rect = Rectangle { position, size };
        for y in 0..size.height as isize {
            for x in 0..size.width as isize {
                buffer.reset(x, y, &rect, &modifiers);
            }
        }
    }

    fn render_bar(
        buffer: &mut Buffer,
        position: Point,
        size: Size,
        value: f32,
        vertical: bool,
        range: Range,
        modifiers: Modifiers,
    ) {
        if modifiers.clear_background {
            Self::clear_background(buffer, position, size, &modifiers);
        }

        let value = clamp(value, range.min, range.max);
        let percentage = (value - range.min) / (range.max - range.min);

        let (height, width) = match vertical {
            true => ((size.height as f32 * percentage) as usize, size.width),
            false => (size.height, (size.width as f32 * percentage) as usize),
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

        if modifiers.clear_background {
            Self::clear_background(buffer, position, size, &modifiers);
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
        text_offset: Offset,
        font_size: Option<usize>,
        modifiers: Modifiers,
        offsets: &mut IntoIter<usize>,
    ) {
        if modifiers.clear_background {
            Self::clear_background(buffer, position, size, &modifiers);
        }

        let mut cursor_x = 0;
        let cursor_y = size.height as isize;

        let offset = offsets.next().expect("Each 'Text' shall have its offset");
        let mut characters = text.chars();
        for _ in 0..offset {
            _ = characters.next();
        }

        let rect = Rectangle { position, size };

        let (font_size, text_offset) = match (font_size, text_offset) {
            (Some(font_size), offset_type) => {
                let offset = self.font_manager.get_offset(font_size, &offset_type);
                (font_size, offset)
            }
            (None, offset_type) => {
                let font_size = self
                    .font_manager
                    .get_font_size(rect.size.height, &offset_type);
                let offset = self.font_manager.get_offset(font_size, &offset_type);
                (font_size, offset)
            }
        };

        for character in characters {
            let character = self.font_manager.get_character(character, font_size);
            let bitmap = &character.bitmap;

            for bitmap_y in 0..bitmap.rows as isize {
                for bitmap_x in 0..bitmap.cols as isize {
                    let x = cursor_x + bitmap_x + bitmap.offset_x;
                    let y = cursor_y + bitmap_y - bitmap.offset_y - text_offset;

                    if x < 0
                        || y < 0
                        || x >= rect.size.width as isize
                        || y >= rect.size.height as isize
                    {
                        continue;
                    }

                    if bitmap.get(bitmap_x as usize, bitmap_y as usize) {
                        buffer.set(x, y, &rect, &modifiers);
                    }
                }
            }

            cursor_x += character.metrics.advance;
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

        match ctx.has_new_data() {
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
        if !text.scrolling {
            return 0;
        }

        let font_size = match (text.font_size, text.text_offset) {
            (Some(font_size), _) => font_size,
            (None, offset_type) => font_manager.get_font_size(text.size.height, &offset_type),
        };
        let text_width = text.size.width;
        let character = font_manager.get_character('a', font_size);
        let char_width = character.metrics.advance as usize;
        let max_characters = text_width / max(char_width, 1);
        let len = text.text.chars().count();
        let tick = ctx.get_tick(&text.text);

        if len <= max_characters {
            ctx.set_wrap(&text.text);
            return 0;
        }

        let max_shifts = len - max_characters;
        let max_ticks = 2 * control.ticks_at_edge + max_shifts * control.ticks_per_move;
        if tick >= max_ticks {
            ctx.set_wrap(&text.text);
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

struct TextData {
    tick: usize,
    can_wrap: bool,
    updated: bool,
}

struct ScrollingTextData {
    contexts: HashMap<ContextKey, HashMap<String, TextData>>,
}

impl ScrollingTextData {
    pub fn new() -> Self {
        Self {
            contexts: HashMap::new(),
        }
    }

    pub fn get_context(&mut self, ctx: ContextKey) -> Context {
        let text_data = self.contexts.entry(ctx).or_insert(HashMap::new());
        Context::new(text_data)
    }
}

struct Context<'a> {
    text_data: &'a mut HashMap<String, TextData>,
    new_data: bool,
}

impl<'a> Context<'a> {
    pub fn new(text_data: &'a mut HashMap<String, TextData>) -> Self {
        Self {
            text_data,
            new_data: false,
        }
    }

    pub fn get_tick(&mut self, key: &String) -> usize {
        match self.text_data.entry(key.clone()) {
            Entry::Occupied(mut data) => {
                let data = data.get_mut();
                if !data.updated {
                    data.tick += 1;
                    data.updated = true;
                }
                data.tick
            }
            Entry::Vacant(data) => {
                self.new_data = true;

                let tick = 0;
                data.insert(TextData {
                    tick,
                    can_wrap: false,
                    updated: true,
                });
                tick
            }
        }
    }

    pub fn set_wrap(&mut self, key: &String) {
        if let Some(data) = self.text_data.get_mut(key) {
            data.can_wrap = true;
        };
    }

    pub fn can_wrap(&self) -> bool {
        self.new_data || self.text_data.iter().all(|(_, data)| data.can_wrap)
    }

    pub fn has_new_data(&self) -> bool {
        self.new_data
    }
}

impl<'a> Drop for Context<'a> {
    fn drop(&mut self) {
        if self.new_data {
            self.text_data.retain(|_, data| data.updated);
        }

        if self.can_wrap() {
            for (_, data) in &mut *self.text_data {
                data.tick = 0;
            }
        }

        for (_, data) in &mut *self.text_data {
            data.can_wrap = false;
            data.updated = false;
        }
    }
}

#[derive(Eq, Hash, PartialEq)]
pub struct ContextKey {
    pub script: usize,
    pub device: usize,
}
