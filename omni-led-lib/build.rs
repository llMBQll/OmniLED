fn main() {
    #[cfg(target_os = "windows")]
    windres::Build::new().compile("../assets/icon.rc").unwrap();
}
