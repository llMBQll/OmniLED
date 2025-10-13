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

use std::env;
use std::path::PathBuf;

pub fn get_app_name() -> &'static str {
    "OmniLED"
}

fn get_exe_name() -> &'static str {
    "omni_led"
}

pub fn get_root_dir() -> PathBuf {
    let root = dirs_next::config_dir().expect("Couldn't get default config directory");
    let root = root.join(get_app_name());
    root
}

pub fn get_bin_dir() -> PathBuf {
    let root = get_root_dir();
    root.join("bin")
}

pub fn get_config_dir() -> PathBuf {
    let root = get_root_dir();
    root.join("config")
}

pub fn get_data_dir() -> PathBuf {
    let root = get_root_dir();
    root.join("data")
}

pub fn get_app_exe_path() -> PathBuf {
    get_bin_dir()
        .join(get_exe_name())
        .with_extension(env::consts::EXE_EXTENSION)
}

pub fn ask_user(message: &str) -> bool {
    loop {
        println!("{message} [Y/N]");

        let response = read_user_input();
        let response = response.trim().to_lowercase();

        if response == "y" || response == "n" {
            break response == "y";
        } else {
            println!("Please choose [Y/N]");
        }
    }
}

pub fn read_user_input() -> String {
    let mut response = String::new();
    std::io::stdin().read_line(&mut response).unwrap();

    if let Some('\n') = response.chars().next_back() {
        response.pop();
    }
    if let Some('\r') = response.chars().next_back() {
        response.pop();
    }

    response
}
