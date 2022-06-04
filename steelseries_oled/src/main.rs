use crate::plugin::Plugin;

mod plugin;

fn main() {
    let plugin = Plugin::new(&String::from("target\\debug\\clock.dll")).expect("Failed to load");
    plugin.update();
}
