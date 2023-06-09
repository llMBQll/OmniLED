use tray_item::{IconSource, TrayItem};

pub struct TrayIcon {
    _tray: TrayItem,
}

impl TrayIcon {
    pub fn new<F>(on_quit: F) -> Self
        where F: Fn() + Send + Sync + 'static
    {
        let mut tray = TrayItem::new(
            "Steelseries OLED",
            IconSource::Resource("white"),
        ).unwrap();

        tray.add_menu_item("Quit", on_quit).unwrap();

        Self {
            _tray: tray,
        }
    }
}