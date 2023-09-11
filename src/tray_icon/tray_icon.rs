use std::sync::atomic::{AtomicBool, Ordering};
use tray_item::{IconSource, TrayItem};

pub struct TrayIcon {
    _tray: TrayItem,
}

impl TrayIcon {
    pub fn new(running: &'static AtomicBool) -> Self {
        let mut tray = TrayItem::new("Steelseries OLED", Self::load_icon()).unwrap();

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
        let path = std::path::Path::new("assets/icons/white.png");

        let (data, width, height) = {
            let image = image::open(&path)
                .expect("Failed to open icon path")
                .into_rgba8();
            let (width, height) = image.dimensions();
            let data = image.into_raw();
            (data, width as i32, height as i32)
        };

        IconSource::Data {
            data,
            width,
            height,
        }
    }
}
