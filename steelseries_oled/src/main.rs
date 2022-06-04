use crate::plugin::Plugin;

mod plugin;

fn main() {
    let rs_plugin = Plugin::new(&String::from("target\\debug\\rust_test.dll")).expect("Failed to load");
    rs_plugin.update();

    let c_plugin = Plugin::new(&String::from("target\\debug\\c_test.dll")).expect("Failed to load");
    c_plugin.update();
}
