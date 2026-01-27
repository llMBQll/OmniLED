use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::{Icon, TrayIconBuilder};
use winit::event_loop::EventLoopProxy;

use crate::OmniLedEvent;

pub enum TrayEvent {
    Config,
    License,
    Quit,
}

pub struct TrayIcon {
    _tray: tray_icon::TrayIcon,
}

impl TrayIcon {
    #[must_use]
    pub fn new(proxy: EventLoopProxy<OmniLedEvent>) -> Self {
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

        MenuEvent::set_event_handler({
            let proxy = proxy.clone();
            Some(move |e: MenuEvent| {
                let event = match e.id.as_ref() {
                    CONFIG_ID => TrayEvent::Config,
                    LICENSE_ID => TrayEvent::License,
                    QUIT_ID => TrayEvent::Quit,
                    _ => return,
                };

                _ = proxy.send_event(OmniLedEvent::Tray(event));
            })
        });

        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip(TITLE)
            .with_icon(Self::load_icon())
            .build()
            .unwrap();

        Self { _tray: tray }
    }

    fn load_icon() -> Icon {
        // TODO load from image

        const SIZE: usize = 64;
        Icon::from_rgba(vec![0xFF; 4 * SIZE * SIZE], SIZE as u32, SIZE as u32).unwrap()
    }
}
