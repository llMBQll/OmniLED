use log::error;
use tray_icon::TrayIconBuilder;
use tray_icon::menu::{Menu, MenuEvent, MenuItem};

use crate::constants::constants::Constants;
use crate::ui::event::Event;
use crate::ui::handler::HandlerProxy;
use crate::ui::icon_image::tray_icon_image;

pub struct TrayIcon {
    _tray: tray_icon::TrayIcon,
}

impl TrayIcon {
    #[must_use]
    pub fn new(constants: Constants, proxy: HandlerProxy) -> Self {
        #[cfg(feature = "dev")]
        const TITLE: &str = "OmniLED (dev)";

        #[cfg(not(feature = "dev"))]
        const TITLE: &str = "OmniLED";

        const CONFIG_ID: &str = "Config";
        const LICENSE_ID: &str = "License";
        const QUIT_ID: &str = "Quit";

        let menu = Menu::with_items(&[
            &MenuItem::with_id(CONFIG_ID, "Config", true, None),
            &MenuItem::with_id(LICENSE_ID, "License", true, None),
            &MenuItem::with_id(QUIT_ID, "Quit", true, None),
        ])
        .unwrap();

        MenuEvent::set_event_handler(Some(move |e: MenuEvent| {
            match e.id.as_ref() {
                CONFIG_ID => {
                    if let Err(err) = opener::reveal(&constants.config_dir) {
                        error!("Failed to reveal config directory: {}", err);
                    }
                }
                LICENSE_ID => {
                    if let Err(err) = opener::reveal(&constants.root_dir.join("LICENSE")) {
                        error!("Failed to reveal license: {}", err);
                    }
                }
                QUIT_ID => proxy.send(Event::Quit),
                _ => return,
            };
        }));

        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip(TITLE)
            .with_icon(tray_icon_image())
            .build()
            .unwrap();

        Self { _tray: tray }
    }
}
