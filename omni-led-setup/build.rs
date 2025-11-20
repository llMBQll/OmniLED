use convert_case::{Case, Casing};
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn name_upper(name: &str) -> String {
    name.to_case(Case::UpperSnake)
}

fn binary_path(name: &str) -> String {
    #[cfg(debug_assertions)]
    const RELEASE_TYPE: &str = "debug";

    #[cfg(not(debug_assertions))]
    const RELEASE_TYPE: &str = "release";

    format!(
        "../../../../../target/{}/{}{}",
        RELEASE_TYPE,
        name,
        env::consts::EXE_SUFFIX
    )
}

fn main() {
    #[cfg(target_os = "windows")]
    windres::Build::new().compile("../assets/icon.rc").unwrap();

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("binaries.rs");
    let mut file = File::create(&dest_path).unwrap();
    for binary in ["omni-led", "audio", "clock", "images", "media", "weather"] {
        writeln!(
            file,
            "pub const {}: &[u8] = include_bytes!(r\"{}\");",
            name_upper(binary),
            binary_path(binary)
        )
        .unwrap();
    }

    println!("cargo::rerun-if-changed=build.rs");
}
