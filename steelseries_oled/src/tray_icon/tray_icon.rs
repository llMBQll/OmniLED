use log::error;
use std::sync::atomic::{AtomicBool, Ordering};
use tray_item::{IconSource, TrayItem};

use crate::constants::constants::Constants;

pub struct TrayIcon {
    _tray: TrayItem,
}

impl TrayIcon {
    pub fn new(running: &'static AtomicBool) -> Self {
        let mut tray = TrayItem::new("Steelseries OLED", Self::load_icon()).unwrap();

        tray.add_menu_item("Settings", || {
            if let Err(err) = opener::reveal(Constants::root_dir().join("settings.lua")) {
                error!("Failed to reveal config directory: {}", err);
            }
        })
        .unwrap();

        tray.add_menu_item("Quit", || running.store(false, Ordering::Relaxed))
            .unwrap();

        Self { _tray: tray }
    }

    #[cfg(target_os = "windows")]
    fn load_icon() -> IconSource {
        IconSource::Resource("white")
    }

    #[cfg(target_os = "linux")]
    fn load_icon() -> IconSource {
        const IMAGE: &[u8] = include_bytes!("../../assets/icons/white.png");

        let image = image::load_from_memory(&IMAGE)
            .expect("Failed to load icon data")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let data = image.into_raw();

        IconSource::Data {
            data,
            width: width as i32,
            height: height as i32,
        }
    }
}
