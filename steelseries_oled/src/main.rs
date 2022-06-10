use freetype as ft;
use freetype::{ffi, RenderMode};
use freetype::Glyph;
use crate::steelseries_api::SteelSeriesAPI;

mod renderer;
mod steelseries_api;

const WIDTH: usize = 128;
const HEIGHT: usize = 40;

fn draw(glyphs: &Vec<(Glyph, char)>) -> [[char; WIDTH]; HEIGHT] {
    let mut screen = [[' ' as char; WIDTH]; HEIGHT];

    let mut cursor_x = 0;
    let cursor_y = 0;

    for (glyph, chr) in glyphs {
        let x = glyph.get_cbox(ffi::FT_GLYPH_BBOX_UNSCALED);
        let x_max = x.xMax;
        let bitmap_glyph = glyph.to_bitmap(RenderMode::Mono, None).unwrap();
        let bitmap = bitmap_glyph.bitmap();

        let left = bitmap_glyph.left();
        let top = bitmap_glyph.top();

        let rows = bitmap.rows();
        let cols = bitmap.width();
        let buffer = bitmap.buffer();

        println!("{}", chr);
        for row in 0..rows {
            for col in 0..cols {
                let x = cursor_x + col + left;
                let y = cursor_y + row + HEIGHT as i32 - top - 2;
                // println!("x - {}, y - {}", x, y);
                if x < 0 || x >= WIDTH as i32 {
                    continue;
                }
                if y < 0 || y >= HEIGHT as i32 {
                    continue;
                }
                if buffer[(row * cols + col) as usize] != 0 {
                    screen[y as usize][x as usize] = '*';
                }
            }
        }

        cursor_x += x_max / 64;
        println!("{}", cursor_x);
    }

    screen
}

fn to_bit_array(array: [[char; WIDTH]; HEIGHT]) -> [u8; WIDTH * HEIGHT / 8] {
    let mut bit_array = [0 as u8; WIDTH * HEIGHT / 8];
    for row in 0..HEIGHT {
        for col in 0..WIDTH {
            if array[row][col] != ' ' {
                bit_array[(row * WIDTH + col) / 8] |= (1 as u8) << ((7 - col % 8) as u8);
            }
        }
    }
    bit_array
}

const HANDLER: &str = r#"(handler \"CLOCK_UPDATE\" (lambda (data) (on-device 'screened show-image: (list-to-bytearray (image-data: (frame: data)))))) (add-event-zone-use-with-specifier \"CLOCK_UPDATE\" \"one\" 'screened)"#;

fn main() {
    let font = "assets/fonts/Roboto-Thin.ttf";
    let library = ft::Library::init().unwrap();
    let face = library.new_face(font, 0).unwrap();

    face.set_char_size(30 * 64, 80 * 64, WIDTH as u32, HEIGHT as u32).unwrap();

    let mut glyphs = Vec::<(Glyph, char)>::new();
    let tmp = vec!('1', '2', ':', '3', '4');
    for x in tmp {
        face.load_char(x as usize, ft::face::LoadFlag::RENDER).unwrap();
        let glyph = face.glyph().get_glyph().unwrap();
        glyphs.push((glyph, x));
    }
    let screen = draw(&glyphs);
    for row in screen {
        for chr in row {
            print!("{}", chr);
        }
        println!();
    }
    let bit_array = to_bit_array(screen);


    let mut api = SteelSeriesAPI::new();
    api.remove_game(r#"{"game":"RUST_STEELSERIES_OLED"}"#).expect("");
    api.game_metadata(r#"{"game":"RUST_STEELSERIES_OLED", "game_display_name":"[Rust] Steelseries OLED", "developer":"MBQ"}"#).expect("");
    api.load_lisp_handlers(format!(r#"{{"game":"RUST_STEELSERIES_OLED", "golisp":"{}"}}"#, HANDLER).as_str()).expect("");
    let update = serde_json::json!({
        "game": "RUST_STEELSERIES_OLED",
        "event": "CLOCK_UPDATE",
        "data": {
            "value": 0,
            "frame": {
                "image-data": bit_array.to_vec()
            }
        }
    });
    println!("{}", serde_json::to_string(&update).unwrap().as_str());
    api.game_event(serde_json::to_string(&update).unwrap().as_str()).expect("");
}
