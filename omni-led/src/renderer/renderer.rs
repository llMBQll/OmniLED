/*
 * OmniLED is a software for displaying data on various OLED devices.
 * Copyright (C) 2024  Michał Bałabanow <m.balabanow@gmail.com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use mlua::Lua;
use num_traits::clamp;
use std::cmp::max;
use std::collections::HashMap;

use crate::common::user_data::UserDataRef;
use crate::renderer::animation::{Animation, Step};
use crate::renderer::buffer::{BitBuffer, Buffer, BufferTrait, ByteBuffer};
use crate::renderer::font_manager::FontManager;
use crate::renderer::images;
use crate::renderer::images::ImageCache;
use crate::script_handler::script_data_types::{
    Bar, Image, MemoryRepresentation, Modifiers, Point, Text, Widget,
};
use crate::script_handler::script_data_types::{Rectangle, Size};
use crate::settings::settings::Settings;

pub struct Renderer {
    font_manager: FontManager,
    image_cache: ImageCache,
    animation_data: AnimationData,
    scrolling_text_settings: ScrollingTextSettings,
    counter: usize,
}

impl Renderer {
    pub fn new(lua: &Lua) -> Self {
        let settings = UserDataRef::<Settings>::load(lua);
        let font_selector = settings.get().font.clone();

        Self {
            font_manager: FontManager::new(font_selector),
            image_cache: ImageCache::new(),
            animation_data: AnimationData::new(),
            scrolling_text_settings: ScrollingTextSettings::new(lua),
            counter: 0,
        }
    }

    pub fn render(
        &mut self,
        key: ContextKey,
        size: Size,
        widgets: Vec<Widget>,
        memory_representation: MemoryRepresentation,
    ) -> (bool, Buffer) {
        self.counter += 1;

        let mut buffer = match memory_representation {
            MemoryRepresentation::BitPerPixel => Buffer::new(BitBuffer::new(size)),
            MemoryRepresentation::BytePerPixel => Buffer::new(ByteBuffer::new(size)),
        };
        let (end_auto_repeat, steps) = self.precalculate_text(&key, &widgets);

        for operation in widgets {
            match operation {
                Widget::Bar(bar) => Self::render_bar(&mut buffer, bar),
                Widget::Image(image) => self.render_image(&mut buffer, image, &key),
                Widget::Text(text) => self.render_text(&mut buffer, text, &steps),
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

    fn render_image(&mut self, buffer: &mut Buffer, widget: Image, key: &ContextKey) {
        if widget.size.width == 0 || widget.size.height == 0 {
            return;
        }

        let image = images::render_image(
            &mut self.image_cache,
            &widget.image,
            widget.size,
            widget.threshold,
            widget.animated,
        );

        let frame = if widget.animated {
            let animation = self
                .animation_data
                .get_image_context(key)
                .entry(widget.image.hash.unwrap())
                .or_insert_with(|| Animation::new(1, 1, image.len(), self.counter));

            let step = animation.step(self.counter);
            if step.can_wrap {
                animation.reset();
            }

            &image[step.offset]
        } else {
            &image[0]
        };

        Self::render_image_impl(buffer, &widget, frame);
    }

    fn render_image_impl(buffer: &mut Buffer, widget: &Image, rendered: &BitBuffer) {
        if widget.modifiers.clear_background {
            Self::clear_background(buffer, widget.position, widget.size, &widget.modifiers);
        }

        let rect = Rectangle {
            position: widget.position,
            size: widget.size,
        };

        for y in 0..rendered.height() {
            for x in 0..rendered.width() {
                if rendered.get(x, y).unwrap() {
                    buffer.set(x as isize, y as isize, &rect, &widget.modifiers);
                }
            }
        }
    }

    fn render_text(&mut self, buffer: &mut Buffer, widget: Text, steps: &HashMap<String, Step>) {
        if widget.modifiers.clear_background {
            Self::clear_background(buffer, widget.position, widget.size, &widget.modifiers);
        }

        let mut cursor_x = 0;
        let cursor_y = widget.size.height as isize;

        let mut characters = widget.text.chars();

        if widget.scrolling {
            let offset = steps
                .get(&widget.text)
                .and_then(|step| Some(step.offset))
                .unwrap_or(0);

            for _ in 0..offset {
                _ = characters.next();
            }
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

    fn precalculate_text(
        &mut self,
        key: &ContextKey,
        widgets: &Vec<Widget>,
    ) -> (bool, HashMap<String, Step>) {
        let ctx = self.animation_data.get_text_context(&key);

        let mut all_can_wrap: bool = true;
        let mut any_new_data: bool = false;

        let steps = widgets
            .iter()
            .filter_map(|widget| match widget {
                Widget::Text(text) => Self::precalculate_single(
                    ctx,
                    &mut self.font_manager,
                    &self.scrolling_text_settings,
                    text,
                    self.counter,
                )
                .and_then(|(new_data, step)| {
                    if new_data {
                        any_new_data = true;
                    }
                    if !step.can_wrap {
                        all_can_wrap = false;
                    }
                    Some((text.text.clone(), step))
                }),
                _ => None,
            })
            .collect();

        *ctx = ctx
            .iter()
            .filter_map(|(text, animation)| {
                if animation.last_update_time() == self.counter {
                    let text = text.clone();
                    let mut animation = animation.clone();
                    if any_new_data || all_can_wrap {
                        animation.reset();
                    }
                    Some((text, animation))
                } else {
                    None
                }
            })
            .collect();

        if any_new_data {
            (false, HashMap::new())
        } else {
            (all_can_wrap, steps)
        }
    }

    fn precalculate_single(
        ctx: &mut HashMap<String, Animation>,
        font_manager: &mut FontManager,
        settings: &ScrollingTextSettings,
        text: &Text,
        counter: usize,
    ) -> Option<(bool, Step)> {
        if !text.scrolling {
            return None;
        }

        let mut new_data = false;
        let animation = ctx.entry(text.text.to_string()).or_insert_with(|| {
            new_data = true;

            let font_size = match (text.font_size, text.text_offset) {
                (Some(font_size), _) => font_size,
                (None, offset_type) => font_manager.get_font_size(text.size.height, &offset_type),
            };
            let text_width = text.size.width;
            let character = font_manager.get_character('a', font_size);
            let char_width = character.metrics.advance as usize;
            let max_characters = text_width / max(char_width, 1);
            let len = text.text.chars().count();

            if len <= max_characters {
                Animation::new(0, 0, 1, counter)
            } else {
                Animation::new(
                    settings.ticks_at_edge,
                    settings.ticks_per_move,
                    len - max_characters + 1,
                    counter,
                )
            }
        });

        Some((new_data, animation.step(counter)))
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

struct AnimationData {
    text_contexts: HashMap<ContextKey, HashMap<String, Animation>>,
    image_contexts: HashMap<ContextKey, HashMap<u64, Animation>>,
}

impl AnimationData {
    pub fn new() -> Self {
        Self {
            text_contexts: HashMap::new(),
            image_contexts: HashMap::new(),
        }
    }

    pub fn get_text_context(&mut self, ctx: &ContextKey) -> &mut HashMap<String, Animation> {
        self.text_contexts
            .entry(ctx.clone())
            .or_insert(HashMap::new())
    }

    pub fn get_image_context(&mut self, ctx: &ContextKey) -> &mut HashMap<u64, Animation> {
        self.image_contexts
            .entry(ctx.clone())
            .or_insert(HashMap::new())
    }
}

#[derive(Eq, Hash, PartialEq, Clone)]
pub struct ContextKey {
    pub script: usize,
    pub device: usize,
}
