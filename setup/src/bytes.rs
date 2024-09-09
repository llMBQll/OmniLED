// Hack to provide variants for debug|release & linux|windows, all of which require a different path
// To top if off `include_bytes!` macro only accepts string literals so a build time generation is
// the only alternative, that I can think of, to writing it 4 times.
include!(concat!(env!("OUT_DIR"), "/binaries.rs"));

pub const APPLICATIONS: &[u8] = include_bytes!("../../config/applications.lua");
pub const DEVICES: &[u8] = include_bytes!("../../config/devices.lua");
pub const SCRIPTS: &[u8] = include_bytes!("../../config/scripts.lua");
pub const SETTINGS: &[u8] = include_bytes!("../../config/settings.lua");
