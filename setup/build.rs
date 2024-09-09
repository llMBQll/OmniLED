use convert_case::{Case, Casing};
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn name_upper(name: &str) -> String {
    name.to_case(Case::ScreamingSnake)
}

fn binary_path(name: &str) -> String {
    // Reference: https://doc.rust-lang.org/beta/cargo/reference/unstable.html#artifact-dependencies
    let key = format!("CARGO_BIN_FILE_{}_{}", name_upper(name), name);
    env::var(key).unwrap()
}

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("binaries.rs");
    let mut file = File::create(&dest_path).unwrap();
    for binary in vec!["steelseries_oled", "audio", "clock", "media", "weather"] {
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
