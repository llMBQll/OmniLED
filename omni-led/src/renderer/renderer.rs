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
use std::hash::{DefaultHasher, Hash, Hasher};
use std::str::Chars;

use crate::common::user_data::UserDataRef;
use crate::renderer::animation::{Animation, State};
use crate::renderer::animation_group::AnimationGroup;
use crate::renderer::buffer::{BitBuffer, Buffer, BufferTrait, ByteBuffer};
use crate::renderer::font_manager::FontManager;
use crate::renderer::images;
use crate::renderer::images::ImageCache;
use crate::script_handler::script_data_types::{
    Bar, Image, MemoryRepresentation, Modifiers, Point, Text, Widget,
};
use crate::script_handler::script_data_types::{Rectangle, Size};
use crate::settings::settings::Settings;

macro_rules! get_animation_settings {
    ($default:expr, $widget:expr) => {
        AnimationSettings {
            ticks_at_edge: $widget
                .animation_ticks_delay
                .unwrap_or($default.ticks_at_edge),
            ticks_per_move: $widget
                .animation_ticks_rate
                .unwrap_or($default.ticks_per_move),
        }
    };
}

pub struct Renderer {
    font_manager: FontManager,
    image_cache: ImageCache,
    animation_settings: AnimationSettings,
}

impl Renderer {
    pub fn new(lua: &Lua) -> Self {
        let settings = UserDataRef::<Settings>::load(lua);
        let font_selector = settings.get().font.clone();

        Self {
            font_manager: FontManager::new(font_selector),
            image_cache: ImageCache::new(),
            animation_settings: AnimationSettings::new(lua),
        }
    }

    pub fn render(
        &mut self,
        animation_groups: &mut HashMap<usize, AnimationGroup>,
        size: Size,
        mut widgets: Vec<Widget>,
        memory_representation: MemoryRepresentation,
    ) -> (State, Buffer) {
        let mut buffer = match memory_representation {
            MemoryRepresentation::BitPerPixel => Buffer::new(BitBuffer::new(size)),
            MemoryRepresentation::BytePerPixel => Buffer::new(ByteBuffer::new(size)),
        };

        self.calculate_animations(animation_groups, &mut widgets);

        for operation in widgets {
            match operation {
                Widget::Bar(bar) => Self::render_bar(&mut buffer, bar),
                Widget::Image(image) => self.render_image(&mut buffer, image, animation_groups),
                Widget::Text(text) => self.render_text(&mut buffer, text, animation_groups),
            }
        }

        animation_groups
            .iter_mut()
            .for_each(|(_, group)| group.sync());

        let state = Self::animation_state(animation_groups);

        (state, buffer)
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

    fn render_image(
        &mut self,
        buffer: &mut Buffer,
        widget: Image,
        animation_groups: &mut HashMap<usize, AnimationGroup>,
    ) {
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
            let hash = widget.image.hash.unwrap();
            let group = Self::get_animation_group(animation_groups, widget.animation_group);
            let animation = group.entry(hash).unwrap();
            let step = animation.step();
            &image[step]
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

    fn render_text(
        &mut self,
        buffer: &mut Buffer,
        widget: Text,
        animation_groups: &mut HashMap<usize, AnimationGroup>,
    ) {
        if widget.size.width == 0 || widget.size.height == 0 {
            return;
        }

        if widget.modifiers.clear_background {
            Self::clear_background(buffer, widget.position, widget.size, &widget.modifiers);
        }

        let mut characters = widget.text.chars();

        if widget.scrolling {
            let hash = widget.hash.unwrap();
            let group = Self::get_animation_group(animation_groups, widget.animation_group);
            let animation = group.entry(hash).unwrap();
            let step = animation.step();

            for _ in 0..step {
                _ = characters.next();
            }
        }

        Self::render_text_impl(buffer, &mut self.font_manager, &widget, characters);
    }

    fn render_text_impl(
        buffer: &mut Buffer,
        font_manager: &mut FontManager,
        widget: &Text,
        characters: Chars,
    ) {
        let rect = Rectangle {
            position: widget.position,
            size: widget.size,
        };

        let font_size = font_manager.get_font_size(widget.font_size, widget.size.height);
        let text_offset = widget
            .text_offset
            .unwrap_or_else(|| font_manager.get_offset(widget.font_size, font_size));

        let mut cursor_x = 0;
        let cursor_y = widget.size.height as isize;

        for character in characters {
            let character = font_manager.get_character(character, font_size);
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

    fn calculate_animations(
        &mut self,
        animation_groups: &mut HashMap<usize, AnimationGroup>,
        widgets: &mut Vec<Widget>,
    ) {
        for widget in widgets {
            match widget {
                Widget::Bar(_) => continue,
                Widget::Image(image) => {
                    if !image.animated {
                        continue;
                    }

                    Self::calculate_animation_hash(&image.image.bytes, &mut image.image.hash);

                    let group = Self::get_animation_group(animation_groups, image.animation_group);
                    group.entry(image.image.hash.unwrap()).or_insert_with(|| {
                        let settings = get_animation_settings!(self.animation_settings, image);
                        let rendered = images::render_image(
                            &mut self.image_cache,
                            &image.image,
                            image.size,
                            image.threshold,
                            image.animated,
                        );
                        Animation::new(
                            settings.ticks_at_edge,
                            settings.ticks_per_move,
                            rendered.len(),
                            image.repeats,
                        )
                    });
                }
                Widget::Text(text) => {
                    if !text.scrolling {
                        continue;
                    }

                    Self::calculate_animation_hash(&text.text, &mut text.hash);

                    let group = Self::get_animation_group(animation_groups, text.animation_group);
                    group.entry(text.hash.unwrap()).or_insert_with(|| {
                        let settings = get_animation_settings!(self.animation_settings, text);
                        let steps = Self::pre_render_text(&mut self.font_manager, text);
                        Animation::new(
                            settings.ticks_at_edge,
                            settings.ticks_per_move,
                            steps,
                            text.repeats,
                        )
                    });
                }
            };
        }

        for (_, group) in animation_groups {
            group.pre_sync();
        }
    }

    fn get_animation_group(
        animation_groups: &mut HashMap<usize, AnimationGroup>,
        number: Option<usize>,
    ) -> &mut AnimationGroup {
        let (number, synced) = match number {
            Some(0) | None => (0, false),
            Some(number) => (number, true),
        };

        animation_groups
            .entry(number)
            .or_insert(AnimationGroup::new(synced))
    }

    fn calculate_animation_hash<H: Hash>(value: &H, hash: &mut Option<u64>) {
        *hash = match hash {
            Some(hash) => Some(*hash),
            None => {
                let mut s = DefaultHasher::new();
                value.hash(&mut s);
                Some(s.finish())
            }
        }
    }

    fn animation_state(animation_groups: &HashMap<usize, AnimationGroup>) -> State {
        let states = animation_groups
            .iter()
            .map(|(_, group)| group.states())
            .flatten()
            .collect::<Vec<_>>();

        let all_finished = states.iter().all(|state| *state == State::Finished);
        let any_in_progress = states.iter().any(|state| *state == State::InProgress);

        if all_finished {
            State::Finished
        } else if any_in_progress {
            State::InProgress
        } else {
            State::CanFinish
        }
    }

    fn pre_render_text(font_manager: &mut FontManager, text: &Text) -> usize {
        let font_size = font_manager.get_font_size(text.font_size, text.size.height);
        let text_width = text.size.width;
        let character = font_manager.get_character('a', font_size);
        let char_width = character.metrics.advance as usize;
        let max_characters = text_width / max(char_width, 1);
        let len = text.text.chars().count();

        if len <= max_characters {
            1
        } else {
            len - max_characters + 1
        }
    }
}

struct AnimationSettings {
    ticks_at_edge: usize,
    ticks_per_move: usize,
}

impl AnimationSettings {
    pub fn new(lua: &Lua) -> Self {
        let settings = UserDataRef::<Settings>::load(lua);
        Self {
            ticks_at_edge: settings.get().animation_ticks_delay,
            ticks_per_move: settings.get().animation_ticks_rate,
        }
    }
}
