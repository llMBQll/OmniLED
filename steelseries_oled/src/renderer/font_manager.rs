use freetype as ft;
use ft::{face::LoadFlag, ffi};
use std::collections::HashMap;
use std::ops::Index;

const FONT_PATH: &str = "steelseries_oled/assets/fonts/CascadiaMonoPL.ttf";

pub struct FontManager {
    _library: ft::Library,
    face: ft::Face,
    face_sizes: HashMap<usize, HashMap<usize, Character>>,
}

impl FontManager {
    pub fn new() -> Self {
        let library = ft::Library::init().unwrap();
        let face = library.new_face(FONT_PATH, 0).unwrap();
        Self {
            _library: library,
            face,
            face_sizes: HashMap::new(),
        }
    }

    pub fn get_character(&mut self, char_code: usize, height: usize) -> &Character {
        let characters = self.face_sizes.entry(height).or_insert(HashMap::new());
        characters.entry(char_code).or_insert_with(|| {
            // TODO proper freetype error checking
            self.face
                .set_pixel_sizes(height as u32, height as u32)
                .unwrap();
            self.face.load_char(char_code, LoadFlag::DEFAULT).unwrap();
            let slot = self.face.glyph();
            let metrics = slot.metrics();
            let glyph = slot.get_glyph().unwrap();
            (metrics, glyph).into()
        })
    }
}

pub struct Character {
    pub metrics: ft::GlyphMetrics,
    pub bounding_box: BoundingBox,
    pub bitmap: Bitmap,
}

impl From<(ft::GlyphMetrics, ft::Glyph)> for Character {
    fn from(params: (ft::GlyphMetrics, ft::Glyph)) -> Self {
        // TODO proper freetype error checking
        let (metrics, glyph) = params;
        Self {
            metrics,
            bounding_box: glyph.get_cbox(ffi::FT_GLYPH_BBOX_UNSCALED).into(),
            bitmap: glyph
                .to_bitmap(ft::RenderMode::Normal, None)
                .unwrap()
                .into(),
        }
    }
}

#[derive(Debug)]
pub struct BoundingBox {
    pub x_min: i32,
    pub x_max: i32,
    pub y_min: i32,
    pub y_max: i32,
}

impl From<ft::BBox> for BoundingBox {
    fn from(bbox: ft::BBox) -> Self {
        Self {
            x_min: bbox.xMin as i32,
            x_max: bbox.xMax as i32,
            y_min: bbox.yMin as i32,
            y_max: bbox.yMax as i32,
        }
    }
}

#[derive(Debug)]
pub struct Bitmap {
    pub top: i32,
    pub left: i32,
    pub rows: usize,
    pub cols: usize,
    buffer: Vec<u8>,
}

impl Bitmap {
    fn get_index(&self, indices: (usize, usize)) -> usize {
        indices.0 * self.cols + indices.1
    }
}

impl From<ft::BitmapGlyph> for Bitmap {
    fn from(bitmap_glyph: ft::BitmapGlyph) -> Self {
        let bitmap = bitmap_glyph.bitmap();
        Self {
            top: bitmap_glyph.top(),
            left: bitmap_glyph.left(),
            rows: bitmap.rows() as usize,
            cols: bitmap.width() as usize,
            buffer: bitmap.buffer().to_vec(),
        }
    }
}

impl Index<(usize, usize)> for Bitmap {
    type Output = u8;

    fn index(&self, indices: (usize, usize)) -> &Self::Output {
        &self.buffer[self.get_index(indices)]
    }
}
