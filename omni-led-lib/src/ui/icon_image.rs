pub fn tray_icon_image() -> tray_icon::Icon {
    let (bytes, width, height) = load_icon();

    tray_icon::Icon::from_rgba(bytes, width, height).unwrap()
}

pub fn window_icon_image() -> winit::window::Icon {
    let (bytes, width, height) = load_icon();

    winit::window::Icon::from_rgba(bytes, width, height).unwrap()
}

fn load_icon() -> (Vec<u8>, u32, u32) {
    const IMAGE: &[u8] = include_bytes!("../../../assets/icons/white.png");

    let image = image::load_from_memory(&IMAGE)
        .expect("Failed to load icon data")
        .into_rgba8();
    let (width, height) = image.dimensions();
    let bytes = image.into_raw();

    (bytes, width, height)
}
