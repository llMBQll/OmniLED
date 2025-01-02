/*
 * OmniLED is a software for displaying data on various OLED devices.
 * Copyright (C) 2025  Michał Bałabanow <m.balabanow@gmail.com>
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

use image::imageops::FilterType;
use std::collections::HashMap;

use crate::renderer::buffer::{BitBuffer, BufferTrait};
use crate::script_handler::script_data_types::{ImageData, Size};

pub type CacheKey = (u64, Size, u8);
pub type ImageCache = HashMap<CacheKey, BitBuffer>;

pub fn render_image<'a>(
    cache: &'a mut ImageCache,
    image: &ImageData,
    size: Size,
    threshold: u8,
) -> &'a BitBuffer {
    cache
        .entry((image.hash.unwrap(), size, threshold))
        .or_insert_with(|| render_image_impl(image, size, threshold))
}

fn render_image_impl(image: &ImageData, size: Size, threshold: u8) -> BitBuffer {
    let image = image::load_from_memory_with_format(&image.bytes, image.format).unwrap();
    let image = image.resize(size.width as u32, size.height as u32, FilterType::Nearest);
    let image = image.into_luma8();

    let mut buffer = BitBuffer::new(size);
    for (x, y, pixel) in image.enumerate_pixels() {
        if pixel[0] >= threshold {
            buffer.set(x as usize, y as usize);
        }
    }
    buffer
}
