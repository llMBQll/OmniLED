use font_kit::font::Font;
use font_kit::properties::Properties;
use font_kit::source::SystemSource;
use freetype::face::LoadFlag;
use freetype::RenderMode;
use log::{debug, error};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

use crate::renderer::bit::Bit;
use crate::renderer::font_selector::FontSelector;

pub struct FontManager {
    _library: freetype::Library,
    face: freetype::Face,
    font_scale: f64,
    offset_scale: f64,
    cache: HashMap<(char, usize), Character>,
}

impl FontManager {
    pub fn new(selector: FontSelector) -> Self {
        let library = freetype::Library::init().unwrap();

        let (data, font_index) = Self::load_font(selector);
        let face = library
            .new_memory_face(data.to_vec(), font_index as isize)
            .unwrap_or_else(|_| panic!("Selected font doesn't have a face at index {}", font_index));

        let ascender = face.ascender() as f64;
        let descender = face.descender() as f64;
        let em_size = face.em_size() as f64;

        let font_scale = em_size / (ascender - descender);
        let offset_scale = (-descender) / (ascender - descender);

        Self {
            _library: library,
            face,
            font_scale,
            offset_scale,
            cache: HashMap::new(),
        }
    }

    pub fn get_font_size(&self, max_height: usize) -> usize {
        let size = max_height as f64 * self.font_scale;
        size.round() as usize
    }

    pub fn get_offset(&self, font_size: usize) -> usize {
        let offset = font_size as f64 * self.offset_scale;
        offset.round() as usize
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

    fn select_font(selector: FontSelector) -> Result<(Font, u32), Box<dyn Error>> {
        match selector.clone() {
            FontSelector::Default => Ok(Self::load_default_font()),
            FontSelector::Filesystem(selector) => {
                let font_index = selector.font_index;
                let font = Font::from_path(&selector.path, font_index)?;
                Ok((font, font_index))
            }
            FontSelector::System(selector) => {
                let names: Vec<_> = selector.names.into_iter().map(|name| name.into()).collect();
                let properties = Properties {
                    style: selector.style.into(),
                    weight: selector.weight.into(),
                    stretch: selector.stretch.into(),
                };
                let handle = SystemSource::new().select_best_match(&names, &properties)?;
                let font = handle.load()?;
                Ok((font, 0 /* will this always be zero? */))
            }
        }
    }

    fn load_font(selector: FontSelector) -> (Arc<Vec<u8>>, u32) {
        let (font, font_index) = match Self::select_font(selector.clone()) {
            Ok((font, font_index)) => (font, font_index),
            Err(err) => {
                error!(
                    "Failed to load font with selector '{:?}': {:?}. Falling back to default",
                    selector, err
                );
                Self::load_default_font()
            }
        };
        debug!(
            "Loaded font: {:?}, {:?}",
            font.full_name(),
            font.properties()
        );

        (font.copy_font_data().unwrap(), font_index)
    }

    fn load_default_font() -> (Font, u32) {
        const DEFAULT_FONT: &[u8] =
            include_bytes!("../../assets/fonts/Meslo/MesloLGLNerdFontMono-Bold.ttf");
        const DEFAULT_FONT_INDEX: u32 = 0;

        let default_font = Arc::new(DEFAULT_FONT.to_vec());
        let font = Font::from_bytes(default_font, DEFAULT_FONT_INDEX).unwrap();

        (font, DEFAULT_FONT_INDEX)
    }
}

pub struct Character {
    pub metrics: Metrics,
    pub bitmap: Bitmap,
}

#[derive(Debug)]
pub struct Metrics {
    // TODO legacy parameters, probably should be deleted
    pub _offset_y: isize,
    pub _offset_x: isize,
    pub advance: isize,
}

impl From<freetype::GlyphMetrics> for Metrics {
    fn from(metrics: freetype::GlyphMetrics) -> Self {
        Self {
            _offset_y: (metrics.horiBearingY >> 6) as isize,
            _offset_x: (metrics.horiBearingX >> 6) as isize,
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
