use font_kit::font::Font;
use font_kit::properties::Properties;
use font_kit::source::SystemSource;
use freetype::face::LoadFlag;
use freetype::{ffi, RenderMode};
use log::{debug, error};
use std::collections::HashMap;
use std::sync::Arc;

use crate::renderer::bit::Bit;
use crate::renderer::font_selector::FontSelector;

pub struct FontManager {
    _library: freetype::Library,
    face: freetype::Face,
    cache: HashMap<(char, usize), Character>,
}

impl FontManager {
    pub fn new(selector: FontSelector) -> Self {
        let library = freetype::Library::init().unwrap();

        let (data, font_index) = Self::load_font(selector);
        let face = library
            .new_memory_face(data.to_vec(), font_index as isize)
            .unwrap();

        Self {
            _library: library,
            face,
            cache: HashMap::new(),
        }
    }

    pub fn get_character(&mut self, character: char, height: usize) -> &Character {
        self.cache.entry((character, height)).or_insert_with(|| {
            self.face
                .set_pixel_sizes(height as u32, height as u32)
                .unwrap();
            self.face
                .load_char(character as usize, LoadFlag::TARGET_MONO)
                .unwrap();
            let slot = self.face.glyph();
            let metrics = slot.metrics();
            let glyph = slot.get_glyph().unwrap();

            Character {
                metrics: metrics.into(),
                bounding_box: glyph.get_cbox(ffi::FT_GLYPH_BBOX_UNSCALED).into(),
                bitmap: glyph.to_bitmap(RenderMode::Mono, None).unwrap().into(),
            }
        })
    }

    fn select_font(selector: FontSelector) -> Result<(Font, u32), Box<dyn std::error::Error>> {
        match selector.clone() {
            FontSelector::Default => Ok(Self::load_default_font()),
            FontSelector::Filesystem(selector) => {
                let font_index = selector.font_index.unwrap_or(0);
                let font = Font::from_path(&selector.path, font_index)?;
                Ok((font, font_index))
            }
            FontSelector::System(selector) => {
                let names: Vec<_> = selector.names.into_iter().map(|name| name.0).collect();
                let properties = Properties {
                    style: selector.style.0,
                    weight: selector.weight.0,
                    stretch: selector.stretch.0,
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
    pub bounding_box: BoundingBox,
    pub bitmap: Bitmap,
}

#[derive(Debug)]
pub struct Metrics {
    pub offset_y: isize,
    pub offset_x: isize,
    pub advance: isize,
}

impl From<freetype::GlyphMetrics> for Metrics {
    fn from(metrics: freetype::GlyphMetrics) -> Self {
        Self {
            offset_y: (metrics.horiBearingY >> 6) as isize,
            offset_x: (metrics.horiBearingX >> 6) as isize,
            advance: (metrics.horiAdvance >> 6) as isize,
        }
    }
}

#[derive(Debug)]
pub struct BoundingBox {
    pub x_min: isize,
    pub x_max: isize,
    pub y_min: isize,
    pub y_max: isize,
}

impl From<freetype::BBox> for BoundingBox {
    fn from(bbox: freetype::BBox) -> Self {
        Self {
            x_min: bbox.xMin as isize,
            x_max: bbox.xMax as isize,
            y_min: bbox.yMin as isize,
            y_max: bbox.yMax as isize,
        }
    }
}

#[derive(Debug)]
pub struct Bitmap {
    pub top: isize,
    pub left: isize,
    pub rows: usize,
    pub cols: usize,
    stride: usize,
    buffer: Vec<u8>,
}

impl Bitmap {
    pub fn get(&self, row: usize, col: usize) -> bool {
        let row_begin = row * self.stride;
        let mut byte = self.buffer[row_begin + col / 8];
        let bit = Bit::new(&mut byte, 7 - col % 8);
        bit.get()
    }
}

impl From<freetype::BitmapGlyph> for Bitmap {
    fn from(bitmap_glyph: freetype::BitmapGlyph) -> Self {
        let bitmap = bitmap_glyph.bitmap();
        Self {
            top: bitmap_glyph.top() as isize,
            left: bitmap_glyph.left() as isize,
            rows: bitmap.rows() as usize,
            cols: bitmap.width() as usize,
            stride: bitmap.pitch() as usize,
            buffer: bitmap.buffer().to_vec(),
        }
    }
}
