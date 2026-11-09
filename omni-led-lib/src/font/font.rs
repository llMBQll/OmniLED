use freetype::{Face, RenderMode, face::LoadFlag};
use mlua::UserData;
use std::collections::HashMap;

use crate::{common::bit::Bit, script_handler::script_data_types::FontSize};

#[derive(Debug)]
pub struct Font {
    face: freetype::Face,
    metrics: FontMetrics,
    cache: HashMap<(char, usize), Character>,
}

impl Font {
    pub fn new(face: Face) -> Self {
        let ascender = face.ascender() as f64;
        let descender = face.descender() as f64;
        let em_size = face.em_size() as f64;

        let full_scale = em_size / (ascender - descender);
        let ascender_only_scale = em_size / ascender;
        let offset_scale = descender.abs() / (ascender - descender);

        Self {
            face,
            metrics: FontMetrics {
                full_scale,
                ascender_only_scale,
                offset_scale,
            },
            cache: HashMap::new(),
        }
    }

    pub fn get_font_size(&self, font_setting: FontSize, max_height: usize) -> usize {
        match font_setting {
            FontSize::Value(value) => value,
            FontSize::Auto => {
                let size = max_height as f64 * self.metrics.full_scale;
                size.round() as usize
            }
            FontSize::AutoUpper => {
                let size = max_height as f64 * self.metrics.ascender_only_scale;
                size.round() as usize
            }
        }
    }

    pub fn get_offset(&self, font_setting: FontSize, actual_font_size: usize) -> isize {
        match font_setting {
            FontSize::Value(_) | FontSize::Auto => {
                let offset = actual_font_size as f64 * self.metrics.offset_scale;
                offset.ceil() as isize
            }
            FontSize::AutoUpper => 1,
        }
    }

    pub fn get_character(&mut self, character: char, font_size: usize) -> &Character {
        self.cache.entry((character, font_size)).or_insert_with(|| {
            self.face
                .set_pixel_sizes(font_size as u32, font_size as u32)
                .unwrap();
            self.face
                .load_char(character as usize, LoadFlag::TARGET_MONO)
                .unwrap();
            let slot = self.face.glyph();
            let metrics = slot.metrics();
            let glyph = slot.get_glyph().unwrap();

            Character {
                metrics: metrics.into(),
                bitmap: glyph.to_bitmap(RenderMode::Mono, None).unwrap().into(),
            }
        })
    }
}

impl UserData for Font {}

#[derive(Debug)]
struct FontMetrics {
    full_scale: f64,
    ascender_only_scale: f64,
    offset_scale: f64,
}

#[derive(Debug)]
pub struct Character {
    pub metrics: Metrics,
    pub bitmap: Bitmap,
}

#[derive(Debug)]
pub struct Metrics {
    pub advance: isize,
}

impl From<freetype::GlyphMetrics> for Metrics {
    fn from(metrics: freetype::GlyphMetrics) -> Self {
        Self {
            advance: (metrics.horiAdvance >> 6) as isize,
        }
    }
}

#[derive(Debug)]
pub struct Bitmap {
    pub offset_x: isize,
    pub offset_y: isize,
    pub rows: usize,
    pub cols: usize,
    stride: usize,
    buffer: Vec<u8>,
}

impl Bitmap {
    pub fn get(&self, x: usize, y: usize) -> bool {
        let row_begin = y * self.stride;
        let mut byte = self.buffer[row_begin + x / 8];
        let bit = Bit::new(&mut byte, 7 - x % 8);
        bit.get()
    }
}

impl From<freetype::BitmapGlyph> for Bitmap {
    fn from(bitmap_glyph: freetype::BitmapGlyph) -> Self {
        let bitmap = bitmap_glyph.bitmap();
        Self {
            offset_x: bitmap_glyph.left() as isize,
            offset_y: bitmap_glyph.top() as isize,
            rows: bitmap.rows() as usize,
            cols: bitmap.width() as usize,
            stride: bitmap.pitch() as usize,
            buffer: bitmap.buffer().to_vec(),
        }
    }
}
