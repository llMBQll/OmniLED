use std::{thread, time};
use std::collections::HashSet;

use crate::lisp_handler::lisp_handler::LispHandler;
use crate::model::display::Display;
use crate::model::position::Position;
use crate::plugin::Plugin;
use crate::renderer::renderer::Renderer;
use crate::steelseries_api::SteelSeriesAPI;

mod lisp_handler;
mod renderer;
mod plugin;
mod model;
mod steelseries_api;

const WIDTH: usize = 128;
const HEIGHT: usize = 40;

const HANDLER: &str = r#"(handler \"UPDATE\" (lambda (data) (on-device 'screened show-image: (list-to-bytearray (image-data: (frame: data)))))) (add-event-zone-use-with-specifier \"CLOCK_UPDATE\" \"one\" 'screened)"#;

fn main() {
    let mut parts = Vec::new();
    // parts.push((
    //     r#" (text (format "{0} {1}.{2:0>2}.{3:0>2}" (nth CLOCK:WeekDay (list "Mon" "Tue" "Wed" "Thu" "Fri" "Sat" "Sun")) CLOCK:MonthDay CLOCK:Month (% CLOCK:Year 100))) "#.to_string(),
    //     Position{ x: 0, y: 0, width: WIDTH, height: 12 }
    // ));
    // parts.push((
    //     r#" (text (format "{0:0>2}:{1:0>2}:{2:0>2}" CLOCK:Hours CLOCK:Minutes CLOCK:Seconds)) "#.to_string(),
    //     Position{ x: 0, y: 12, width: WIDTH, height: 12 }
    // ));
    // parts.push((
    //     r#" (bar (/ (* CLOCK:Seconds 100) 59)) "#.to_string(),
    //     Position{ x: 0, y: 25, width: WIDTH, height: 13 }
    // ));
    parts.push((
        r#" (text (format "{0:0>2}:{1:0>2}" CLOCK:Hours CLOCK:Minutes)) "#.to_string(),
        Position{ x: 25, y: 0, width: WIDTH - 25, height: 36 }
    ));
    parts.push((
        r#" (bar (/ (* CLOCK:Seconds 100) 59)) "#.to_string(),
        Position{ x: 0, y: 38, width: WIDTH, height: 2 }
    ));

    let name = String::from("__handler0");

    let mut sensitivity_list = HashSet::new();
    sensitivity_list.insert("CLOCK:Seconds".to_string());

    let clock_display = Display::new(name, parts, sensitivity_list);

    let c_plugin = Plugin::new(&String::from("target\\debug\\clock.dll")).expect("Failed to load");
    let _ = c_plugin.types();

    let mut handler = LispHandler::new();
    let mut displays = Vec::new();
    displays.push(clock_display);

    let mut renderer = Renderer::new(HEIGHT, WIDTH);
    let mut api = SteelSeriesAPI::new();
    api.remove_game(r#"{"game":"RUST_STEELSERIES_OLED"}"#).expect("");
    api.game_metadata(r#"{"game":"RUST_STEELSERIES_OLED", "game_display_name":"[Rust] Steelseries OLED", "developer":"MBQ"}"#).expect("");
    api.load_lisp_handlers(format!(r#"{{"game":"RUST_STEELSERIES_OLED", "golisp":"{}"}}"#, HANDLER).as_str()).expect("");

    handler.register(displays).expect("Register failed");

    let duration = time::Duration::from_millis(1000);
    loop {
        // let begin = time::Instant::now();

        let map = c_plugin.update();
        let mut plugins = Vec::new();
        plugins.push((c_plugin.display_name(), map));
        let results = handler.update(&plugins);

        match results {
            Ok(results) => {
                match results.len() {
                    0 => {
                        // println!("{}", "No update")
                    }
                    _ => {
                        let update = serde_json::json!({
                            "game": "RUST_STEELSERIES_OLED",
                            "event": "UPDATE",
                            "data": {
                                "value": 0,
                                "frame": {
                                    "image-data": renderer.render(results)
                                }
                            }
                        });
                        // let end = time::Instant::now();
                        api.game_event(serde_json::to_string(&update).unwrap().as_str()).expect("");
                        // let end2 = time::Instant::now();
                        // println!("Time: {:0<5.3}[ms]", (end - begin).as_micros() as f32 / 1000.0);
                        // println!("Time: {:0<5.3}[ms]", (end2 - begin).as_micros() as f32 / 1000.0);
                    }
                }
            }
            Err(err) => {
                println!("{}", err);
            }
        }

        thread::sleep(duration);
    }
}
