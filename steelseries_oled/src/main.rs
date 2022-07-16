// use std::{thread, time};
// use std::fs::File;
//
// use crate::lisp_handler::lisp_handler::LispHandler;
// use crate::model::display::Display;
// use crate::model::position::Position;
// use crate::plugin::plugin::Plugin;
// use crate::renderer::renderer::Renderer;
// use crate::steelseries_api::SteelSeriesAPI;
//
// mod lisp_handler;
// mod renderer;
// mod plugin;
// mod model;
// mod steelseries_api;
//
// const WIDTH: usize = 128;
// const HEIGHT: usize = 40;
//
// const HANDLER: &str = r#"(handler \"UPDATE\" (lambda (data) (on-device 'screened show-image: (list-to-bytearray (image-data: (frame: data)))))) (add-event-zone-use-with-specifier \"CLOCK_UPDATE\" \"one\" 'screened)"#;
//
// fn path(name: &str) -> String {
//     #[cfg(debug_assertions)]
//     let str = format!("target\\debug\\{}", name);
//
//     #[cfg(not(debug_assertions))]
//     let str = format!("target\\release\\{}", name);
//
//     println!("Loading: {}", str);
//
//     str
// }
//
// fn main() {
//     // let mut file = File::open("displays.json").unwrap();
//     // file.write_all(serde_json::to_string_pretty(&displays).unwrap().as_ref()).unwrap();
//
//     let mut file = File::open("displays.json").unwrap();
//     let displays: Vec<Display> = serde_json::from_reader(&mut file).unwrap();
//
//     let spotify_plugin = Plugin::new(&path("spotify.dll")).expect("Failed to load");
//     let clock_plugin = Plugin::new(&path("clock.dll")).expect("Failed to load");
//     let audio_plugin = Plugin::new(&path("audio.dll")).expect("Failed to load");
//     let _ = clock_plugin.types();
//
//     let mut handler = LispHandler::new();
//     let mut renderer = Renderer::new(HEIGHT, WIDTH);
//     let mut api = SteelSeriesAPI::new();
//
//     match api.remove_game(r#"{"game":"RUST_STEELSERIES_OLED"}"#) {
//         _ => { }
//     }
//     api.game_metadata(r#"{"game":"RUST_STEELSERIES_OLED", "game_display_name":"[Rust] Steelseries OLED", "developer":"MBQ"}"#).expect("/game_metadata");
//     api.load_lisp_handlers(format!(r#"{{"game":"RUST_STEELSERIES_OLED", "golisp":"{}"}}"#, HANDLER).as_str()).expect("/load_lisp_handlers");
//
//     handler.register(displays).expect("Register failed");
//
//     let duration = time::Duration::from_millis(50);
//     let mut last_update = time::Instant::now();
//     const MAX_UPDATE_INTERVAL: time::Duration = time::Duration::from_secs(10);
//     loop {
//         let update_begin = time::Instant::now();
//
//         let mut plugins = Vec::new();
//         plugins.push((audio_plugin.name(), audio_plugin.update()));
//         plugins.push((clock_plugin.name(), clock_plugin.update()));
//         plugins.push((spotify_plugin.name(), spotify_plugin.update()));
//         let results = handler.update(&plugins, duration);
//
//         match results {
//             Ok(results) => {
//                 match results.len() {
//                     0 => {
//                         if update_begin.saturating_duration_since(last_update) > MAX_UPDATE_INTERVAL {
//                             last_update = update_begin;
//                             api.game_heartbeat(r#"{"game": "RUST_STEELSERIES_OLED"}"#).expect("/game_heartbeat");
//                         }
//                     }
//                     _ => {
//                         let update = serde_json::json!({
//                             "game": "RUST_STEELSERIES_OLED",
//                             "event": "UPDATE",
//                             "data": {
//                                 "value": 0,
//                                 "frame": {
//                                     "image-data": renderer.render(results)
//                                 }
//                             }
//                         });
//                         api.game_event(serde_json::to_string(&update).unwrap().as_str()).expect("/game_event");
//                     }
//                 }
//             }
//             Err(err) => {
//                 println!("{}", err);
//             }
//         }
//
//         let update_end = time::Instant::now();
//         let update_duration = update_end - update_begin;
//         thread::sleep(duration.saturating_sub(update_duration));
//     }
// }
fn main() {

}