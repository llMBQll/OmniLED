use log::error;
use mlua::Lua;
use std::sync::atomic::{AtomicBool, Ordering};
use tray_item::{IconSource, TrayItem};

use crate::common::user_data::UserDataRef;
use crate::constants::constants::Constants;

pub struct TrayIcon {
    _tray: TrayItem,
}

impl TrayIcon {
    #[must_use]
    pub fn new(lua: &Lua, running: &'static AtomicBool) -> Self {
        #[cfg(feature = "dev")]
        const TITLE: &str = "OmniLED (dev)";

        #[cfg(not(feature = "dev"))]
        const TITLE: &str = "OmniLED";

        let constants = UserDataRef::<Constants>::load(lua);
        let config_dir = constants.get().config_dir.clone();
        let license = config_dir.join("LICENSE");

        let mut tray = TrayItem::new(TITLE, Self::load_icon()).unwrap();

        tray.add_menu_item("Config", move || {
            if let Err(err) = opener::reveal(&config_dir) {
                error!("Failed to reveal config directory: {}", err);
            }
        })
        .unwrap();

        tray.add_menu_item("License", move || {
            if let Err(err) = opener::reveal(&license) {
                error!("Failed to reveal license: {}", err);
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
        const IMAGE: &[u8] = include_bytes!("../../../assets/icons/white.png");

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
