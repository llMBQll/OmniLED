/*
 * OmniLED is a software for displaying data on various OLED devices.
 * Copyright (C) 2024  Michał Bałabanow <m.balabanow@gmail.com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

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
