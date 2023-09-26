fn main() {
    #[cfg(target_os = "windows")]
    windres::Build::new()
        .compile("assets/tray_icon.rc")
        .unwrap();
}
