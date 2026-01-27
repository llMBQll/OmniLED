use std::fmt::{Debug, Formatter};

pub mod app_loader;
pub mod common;
pub mod constants;
pub mod devices;
pub mod events;
pub mod keyboard;
pub mod logging;
pub mod renderer;
pub mod script_handler;
pub mod semaphore;
pub mod server;
pub mod settings;
pub mod tray_icon;

pub enum OmniLedEvent {
    NewScreen(devices::emulator::emulator::EmulatorHandle),
    Update,
    Tray(tray_icon::tray_icon::TrayEvent),
}

impl Debug for OmniLedEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("OmniLedEvent")
    }
}
