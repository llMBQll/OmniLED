use crate::plugin::Plugin;
use std::{thread, time};
use crate::steelseries_api::SteelSeriesAPI;

mod plugin;
mod steelseries_api;

const HANDLER: &str = r#"(handler \"CLOCK_UPDATE\" (lambda (data) (let ((x (value: data))) (on-device 'screened show-text-on-zone: x one:)))) (add-event-zone-use-with-specifier \"CLOCK_UPDATE\" \"one\" 'screened)"#;

fn main() {
    let mut api = SteelSeriesAPI::new();

    api.remove_game(r#"{"game":"RUST_STEELSERIES_OLED"}"#).expect("");
    api.game_metadata(r#"{"game":"RUST_STEELSERIES_OLED", "game_display_name":"[Rust] Steelseries OLED", "developer":"MBQ"}"#).expect("");
    api.load_lisp_handlers(format!(r#"{{"game":"RUST_STEELSERIES_OLED", "golisp":"{}"}}"#, HANDLER).as_str()).expect("");

    let c_plugin = Plugin::new(&String::from("target\\debug\\clock.dll")).expect("Failed to load");
    println!("{}", c_plugin.display_name());
    println!("{:?}", c_plugin.types());

    let duration = time::Duration::from_millis(200);
    loop {
        let begin = time::Instant::now();
        let map = c_plugin.update();
        let end = time::Instant::now();
        println!("'{:?}' - {}", map, (end - begin).as_micros());
        match map {
            Some(map) => { api.game_event(format!(r#"{{"game":"RUST_STEELSERIES_OLED", "event": "CLOCK_UPDATE", "data": {{"value": "{}"}}}}"#, map["Seconds"]).as_str()).unwrap(); },
            None => {}
        };
        thread::sleep(duration);
    }
}
