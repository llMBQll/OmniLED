use crate::plugin::Plugin;
use std::{thread, time};

mod plugin;
mod steelseries_api;

fn main() {
    let c_plugin = Plugin::new(&String::from("target\\debug\\clock.dll")).expect("Failed to load");
    println!("{}", c_plugin.display_name());
    println!("{:?}", c_plugin.types());

    let duration = time::Duration::from_millis(200);
    loop {
        let begin = time::Instant::now();
        let x = c_plugin.update();
        let end = time::Instant::now();
        println!("'{:?}' - {}", x, (end - begin).as_micros());
        thread::sleep(duration);
    }
}
