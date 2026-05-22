use convert_case::{Case, Casing};
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn root_path() -> &'static str {
    #[cfg(debug_assertions)]
    const RELEASE_TYPE: &str = "../../../../../target/debug";

    #[cfg(not(debug_assertions))]
    const RELEASE_TYPE: &str = "../../../../../target/release";

    RELEASE_TYPE
}

enum OmniLedBinary {
    Exe(&'static str),
    Dll(&'static str),
}

impl OmniLedBinary {
    fn name_upper(&self) -> String {
        let name = match self {
            OmniLedBinary::Exe(name) => name,
            OmniLedBinary::Dll(name) => name,
        };
        name.to_case(Case::UpperSnake)
    }

    fn path(&self) -> String {
        match self {
            OmniLedBinary::Exe(name) => {
                format!("{}/{}{}", root_path(), name, env::consts::EXE_SUFFIX)
            }
            OmniLedBinary::Dll(name) => {
                format!(
                    "{}/{}{}{}",
                    root_path(),
                    env::consts::DLL_PREFIX,
                    name,
                    env::consts::DLL_SUFFIX
                )
            }
        }
    }
}

macro_rules! exe {
    ($name:literal) => {
        OmniLedBinary::Exe($name)
    };
}

macro_rules! dll {
    ($name:literal) => {
        OmniLedBinary::Dll($name)
    };
}

fn main() {
    #[cfg(target_os = "windows")]
    windres::Build::new().compile("../assets/icon.rc").unwrap();

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("binaries.rs");
    let mut file = File::create(&dest_path).unwrap();

    for bin in [
        exe!("omni-led"),
        dll!("audio"),
        dll!("clock"),
        dll!("images"),
        dll!("media"),
        dll!("system"),
        dll!("weather"),
    ] {
        writeln!(
            file,
            "pub const {}: &[u8] = include_bytes!(r\"{}\");",
            bin.name_upper(),
            bin.path()
        )
        .unwrap();
    }

    println!("cargo::rerun-if-changed=build.rs");
}
