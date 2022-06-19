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

fn clock_display() -> Display {
    let mut parts = Vec::new();
    parts.push((
        r#" (text (format "{0:0>2}" CLOCK:Hours)) "#.to_string(),
        Position{ x: 20, y: 4, width: WIDTH - 20, height: 40 }
    ));
    parts.push((
        r#" (text (format "{0:0>2}" CLOCK:Minutes)) "#.to_string(),
        Position{ x: 67, y: 4, width: WIDTH - 65, height: 35 }
    ));
    parts.push((
        r#" (text (format "{0:0>2} {1}" CLOCK:MonthDay (nth CLOCK:Month (list "Jan" "Feb" "Mar" "Apr" "May" "Jun" "Jul" "Aug" "Sep" "Oct" "Nov" "Dec")))) "#.to_string(),
        Position{ x: 69, y: 29, width: WIDTH - 65, height: 20 }
    ));
    parts.push((
        r#" (bar (/ (* CLOCK:Seconds 100) 59)) "#.to_string(),
        Position{ x: 0, y: 38, width: WIDTH, height: 2 }
    ));

    let name = String::from("__handler0");

    let mut sensitivity_list = HashSet::new();
    sensitivity_list.insert("CLOCK:Seconds".to_string());

    Display::new(name, parts, sensitivity_list)
}

fn audio_display() -> Display {
    let mut parts = Vec::new();
    parts.push((
        r#" (if AUDIO:IsMuted (text "MUTED") (text (format "[{0: >3}" AUDIO:Volume))) "#.to_string(),
        Position{ x: 0, y: 4, width: WIDTH, height: 40 }
    ));
    parts.push((
        r#" (bar AUDIO:Volume) "#.to_string(),
        Position{ x: 0, y: 38, width: WIDTH, height: 2 }
    ));

    let name = String::from("__handler1");

    let mut sensitivity_list = HashSet::new();
    sensitivity_list.insert("AUDIO:Volume".to_string());
    sensitivity_list.insert("AUDIO:IsMuted".to_string());

    Display::new(name, parts, sensitivity_list)
}

fn main() {
    let c_plugin = Plugin::new(&String::from("target\\release\\clock.dll")).expect("Failed to load");
    let r_plugin = Plugin::new(&String::from("target\\release\\audio.dll")).expect("Failed to load");
    let _ = c_plugin.types();
    let _ = r_plugin.types();

    let mut handler = LispHandler::new();
    let mut displays = Vec::new();
    displays.push(audio_display());
    displays.push(clock_display());

    let mut renderer = Renderer::new(HEIGHT, WIDTH);
    let mut api = SteelSeriesAPI::new();
    api.remove_game(r#"{"game":"RUST_STEELSERIES_OLED"}"#).expect("");
    api.game_metadata(r#"{"game":"RUST_STEELSERIES_OLED", "game_display_name":"[Rust] Steelseries OLED", "developer":"MBQ"}"#).expect("");
    api.load_lisp_handlers(format!(r#"{{"game":"RUST_STEELSERIES_OLED", "golisp":"{}"}}"#, HANDLER).as_str()).expect("");

    handler.register(displays).expect("Register failed");

    let duration = time::Duration::from_millis(50);
    loop {
        let update_begin = time::Instant::now();

        let mut plugins = Vec::new();
        plugins.push((r_plugin.name(), r_plugin.update()));
        plugins.push((c_plugin.name(), c_plugin.update()));
        let results = handler.update(&plugins, duration);

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

        let update_end = time::Instant::now();
        let update_duration = update_end - update_begin;
        thread::sleep(duration.saturating_sub(update_duration));
    }
}
