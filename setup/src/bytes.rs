include!(concat!(env!("OUT_DIR"), "/binaries.rs"));

pub const APPLICATIONS: &[u8] = include_bytes!("../../config/applications.lua");
pub const DEVICES: &[u8] = include_bytes!("../../config/devices.lua");
pub const SCRIPTS: &[u8] = include_bytes!("../../config/scripts.lua");
pub const SETTINGS: &[u8] = include_bytes!("../../config/settings.lua");
