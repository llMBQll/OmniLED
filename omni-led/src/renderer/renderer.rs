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
    Bar, Image, MemoryRepresentation, Modifiers, Point, Text, Widget,
};
use crate::script_handler::script_data_types::{Rectangle, Size};
use crate::settings::settings::Settings;

pub struct Renderer {
    font_manager: FontManager,
    scrolling_text_data: ScrollingTextData,
    scrolling_text_settings: ScrollingTextSettings,
}

impl Renderer {
    pub fn new(lua: &Lua) -> Self {
        let settings = UserDataRef::<Settings>::load(lua);
        let font_selector = settings.get().font.clone();

        Self {
            font_manager: FontManager::new(font_selector),
            scrolling_text_data: ScrollingTextData::new(),
            scrolling_text_settings: ScrollingTextSettings::new(lua),
        }
    }

    pub fn render(
        &mut self,
        ctx: ContextKey,
        size: Size,
        widgets: Vec<Widget>,
        memory_representation: MemoryRepresentation,
    ) -> (bool, Buffer) {
        let mut buffer = match memory_representation {
            MemoryRepresentation::BitPerPixel => Buffer::new(BitBuffer::new(size)),
            MemoryRepresentation::BytePerPixel => Buffer::new(ByteBuffer::new(size)),
        };
        let (end_auto_repeat, text_offsets) = self.precalculate_text(ctx, &widgets);
        let mut text_offsets = text_offsets.into_iter();

        for operation in widgets {
            match operation {
                Widget::Bar(bar) => Self::render_bar(&mut buffer, bar),
                Widget::Image(image) => Self::render_image(&mut buffer, image),
                Widget::Text(text) => self.render_text(&mut buffer, text, &mut text_offsets),
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

    fn render_bar(buffer: &mut Buffer, widget: Bar) {
        if widget.modifiers.clear_background {
            Self::clear_background(buffer, widget.position, widget.size, &widget.modifiers);
        }

        let value = clamp(widget.value, widget.range.min, widget.range.max);
        let percentage = (value - widget.range.min) / (widget.range.max - widget.range.min);

        let (height, width) = match widget.vertical {
            true => (
                (widget.size.height as f32 * percentage).round() as usize,
                widget.size.width,
            ),
            false => (
                widget.size.height,
                (widget.size.width as f32 * percentage).round() as usize,
            ),
        };

        let rect = Rectangle {
            position: widget.position,
            size: widget.size,
        };
        for y in 0..height as isize {
            for x in 0..width as isize {
                buffer.set(x, y, &rect, &widget.modifiers);
            }
        }
    }

    fn render_image(buffer: &mut Buffer, widget: Image) {
        if widget.size.width == 0 || widget.size.height == 0 {
            return;
        }

        if widget.modifiers.clear_background {
            Self::clear_background(buffer, widget.position, widget.size, &widget.modifiers);
        }

        let x_factor = widget.image.size.width as f64 / widget.size.width as f64;
        let y_factor = widget.image.size.height as f64 / widget.size.height as f64;

        let rect = Rectangle {
            position: widget.position,
            size: widget.size,
        };
        for y in 0..widget.size.height as isize {
            for x in 0..widget.size.width as isize {
                // Use nearest neighbour interpolation for now as it's the quickest to implement
                // TODO allow specifying scaling algorithm as modifier
                // TODO potentially cache scaled images

                let image_x = (x as f64 * x_factor).round() as usize;
                let image_x = image_x.clamp(0, widget.image.size.width - 1);

                let image_y = (y as f64 * y_factor).round() as usize;
                let image_y = image_y.clamp(0, widget.image.size.height - 1);

                let index = image_y * widget.image.size.width + image_x;
                if widget.image.bytes[index] != 0 {
                    buffer.set(x, y, &rect, &widget.modifiers);
                }
            }
        }
    }

    fn render_text(&mut self, buffer: &mut Buffer, widget: Text, offsets: &mut IntoIter<usize>) {
        if widget.modifiers.clear_background {
            Self::clear_background(buffer, widget.position, widget.size, &widget.modifiers);
        }

        let mut cursor_x = 0;
        let cursor_y = widget.size.height as isize;

        let offset = offsets.next().expect("Each 'Text' shall have its offset");
        let mut characters = widget.text.chars();
        for _ in 0..offset {
            _ = characters.next();
        }

        let rect = Rectangle {
            position: widget.position,
            size: widget.size,
        };

        let (font_size, text_offset) = match (widget.font_size, widget.text_offset) {
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
                        buffer.set(x, y, &rect, &widget.modifiers);
                    }
                }
            }

            cursor_x += character.metrics.advance;
            if cursor_x > rect.size.width as isize {
                break;
            }
        }
    }

    fn precalculate_text(&mut self, ctx: ContextKey, widgets: &Vec<Widget>) -> (bool, Vec<usize>) {
        let mut ctx = self.scrolling_text_data.get_context(ctx);

        let offsets: Vec<usize> = widgets
            .iter()
            .filter_map(|widget| match widget {
                Widget::Text(text) => Some(Self::precalculate_single(
                    &mut ctx,
                    &mut self.font_manager,
                    &self.scrolling_text_settings,
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
        settings: &ScrollingTextSettings,
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
        let max_ticks = 2 * settings.ticks_at_edge + max_shifts * settings.ticks_per_move;
        if tick >= max_ticks {
            ctx.set_wrap(&text.text);
        }

        if tick <= settings.ticks_at_edge {
            0
        } else if tick >= settings.ticks_at_edge + max_shifts * settings.ticks_per_move {
            max_shifts
        } else {
            (tick - settings.ticks_at_edge) / settings.ticks_per_move
        }
    }
}

struct ScrollingTextSettings {
    ticks_at_edge: usize,
    ticks_per_move: usize,
}

impl ScrollingTextSettings {
    pub fn new(lua: &Lua) -> Self {
        let settings = UserDataRef::<Settings>::load(lua);
        Self {
            ticks_at_edge: settings.get().text_ticks_scroll_delay,
            ticks_per_move: settings.get().text_ticks_scroll_rate,
        }
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
