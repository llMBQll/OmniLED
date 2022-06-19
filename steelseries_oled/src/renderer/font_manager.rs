use std::collections::HashMap;
use std::ops::Index;
use freetype as ft;
use ft::{ffi, face::LoadFlag};

const FONT_PATH: &str = "assets/fonts/CascadiaMonoPL.ttf";

pub struct FontManager {
    _library: ft::Library,
    face: ft::Face,
    face_sizes: HashMap<usize, HashMap<usize, Character>>
}

impl FontManager {
    pub fn new() -> Self {
        let library = ft::Library::init().unwrap();
        let face = library.new_face(FONT_PATH, 0).unwrap();
        Self {
            _library: library,
            face,
            face_sizes: HashMap::new()
        }
    }

    pub fn get_character(&mut self, char_code: usize, height: usize) -> &Character {
        let characters = self.face_sizes.entry(height).or_insert(HashMap::new());
        characters.entry(char_code).or_insert_with(|| {
            // TODO proper freetype error checking
            self.face.set_char_size(0, (height * 2 * 60) as isize, 0, height as u32).unwrap();
            self.face.load_char(char_code, LoadFlag::RENDER).unwrap();
            let character: Character = self.face.glyph().get_glyph().unwrap().into();
            // println!("{} - height: {}, actual: {}", char::from_u32(char_code as u32).unwrap(), height, character.bitmap.rows);
            character
        })
    }
//
//     fn map_height_to_size(height: usize) -> usize {
//         if height == 0 {
//             return 0;
//         }
//         // [1, inf] -> [0, 39]
//         let height = std::cmp::min(height, 40) - 1;
//
//         const ARRAY: [usize; 40] = [
//             1 * 64,  // 1
//             2 * 60,  // 2
//             3 * 64,  // 3
//             6 * 64,  // 4
//             7 * 64,  // 5
//             9 * 64,  // 6
//             11 * 64, // 7
//             13 * 64, // 8
//             14 * 64, // 9
//             16 * 64, // 10
//             18 * 64, // 11
//             21 * 64, // 12
//             22 * 64, // 13
//             24 * 64, // 14
//             25 * 64, // 15
//             27 * 64, // 16
//             28 * 64, // 17
//             30 * 64, // 18
//             32 * 64, // 19
//             36 * 64, // 20
//             38 * 64, // 21
//             40 * 64, // 22
//             41 * 64, // 23
//             44 * 64, // 24
//             46 * 64, // 25
//             48 * 64, // 26
//             50 * 64, // 27
//             51 * 64, // 28
//             52 * 64, // 29
//             54 * 64, // 30
//             57 * 64, // 31
//             59 * 64, // 32
//             61 * 64, // 33
//             62 * 64, // 34
//             64 * 64, // 35
//             66 * 64, // 36
//             68 * 64, // 37
//             68 * 64, // 38
//             71 * 64, // 39
//             73 * 64, // 40
//         ];
//
//         ARRAY[height]
//     }
}

pub struct Character {
    pub bounding_box: BoundingBox,
    pub bitmap: Bitmap,
}

impl From<ft::Glyph> for Character {
    fn from(glyph: ft::Glyph) -> Self {
        // TODO proper freetype error checking
        Self {
            bounding_box: glyph.get_cbox(ffi::FT_GLYPH_BBOX_UNSCALED).into(),
            bitmap: glyph.to_bitmap(ft::RenderMode::Mono, None).unwrap().into(),
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
            x_min: bbox.xMin,
            x_max: bbox.xMax,
            y_min: bbox.yMin,
            y_max: bbox.yMax,
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
            buffer: bitmap.buffer().to_vec()
        }
    }
}

impl Index<(usize, usize)> for Bitmap {
    type Output = u8;

    fn index(&self, indices: (usize, usize)) -> &Self::Output {
        &self.buffer[self.get_index(indices)]
    }
}