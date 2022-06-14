use crate::renderer::renderer::Renderer;
use crate::steelseries_api::SteelSeriesAPI;

use std::{thread, time};

mod renderer;
mod steelseries_api;

const WIDTH: usize = 128;
const HEIGHT: usize = 40;

const HANDLER: &str = r#"(handler \"CLOCK_UPDATE\" (lambda (data) (on-device 'screened show-image: (list-to-bytearray (image-data: (frame: data)))))) (add-event-zone-use-with-specifier \"CLOCK_UPDATE\" \"one\" 'screened)"#;

fn main() {
    let mut r = Renderer::new(HEIGHT, WIDTH);

    let mut api = SteelSeriesAPI::new();
    api.remove_game(r#"{"game":"RUST_STEELSERIES_OLED"}"#).expect("");
    api.game_metadata(r#"{"game":"RUST_STEELSERIES_OLED", "game_display_name":"[Rust] Steelseries OLED", "developer":"MBQ"}"#).expect("");
    api.load_lisp_handlers(format!(r#"{{"game":"RUST_STEELSERIES_OLED", "golisp":"{}"}}"#, HANDLER).as_str()).expect("");

    let duration = time::Duration::from_millis(250);
    loop {
        for percent in 0..100 {
            let update = serde_json::json!({
                "game": "RUST_STEELSERIES_OLED",
                "event": "CLOCK_UPDATE",
                "data": {
                    "value": 0,
                    "frame": {
                        "image-data": r.render(percent)
                    }
                }
            });
            // println!("{}", serde_json::to_string(&update).unwrap().as_str());
            api.game_event(serde_json::to_string(&update).unwrap().as_str()).expect("");
            thread::sleep(duration);
        }
    }
}
