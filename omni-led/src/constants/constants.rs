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

use mlua::{chunk, Lua};
use std::path::PathBuf;
use std::{
    env::consts::{EXE_EXTENSION, EXE_SUFFIX, OS},
    path::MAIN_SEPARATOR_STR,
};

use crate::create_table;

pub struct Constants;

impl Constants {
    pub fn load(lua: &Lua) {
        let applications_dir = Self::applications_dir();
        let applications_dir = applications_dir.to_str().unwrap();

        let platform = create_table!(lua, {
            ApplicationsDir = $applications_dir,
            ExeExtension = $EXE_EXTENSION,
            ExeSuffix = $EXE_SUFFIX,
            PathSeparator = $MAIN_SEPARATOR_STR,
            Os = $OS,
        });
        lua.globals().set("PLATFORM", platform).unwrap();
    }

    #[cfg(feature = "dev")]
    pub fn root_dir() -> PathBuf {
        let root_dir = PathBuf::from(".");
        root_dir
    }

    #[cfg(not(feature = "dev"))]
    pub fn root_dir() -> PathBuf {
        let root_dir = dirs_next::config_dir().expect("Couldn't get default config directory");
        let root_dir = root_dir.join("OmniLED");
        root_dir
    }

    #[cfg(feature = "dev")]
    pub fn applications_dir() -> PathBuf {
        #[cfg(debug_assertions)]
        const PATH: &str = "debug";

        #[cfg(not(debug_assertions))]
        const PATH: &str = "release";

        Self::root_dir().join("target").join(PATH)
    }

    #[cfg(not(feature = "dev"))]
    pub fn applications_dir() -> PathBuf {
        Self::root_dir().join("bin")
    }

    pub fn config_dir() -> PathBuf {
        Self::root_dir().join("config")
    }

    pub fn data_dir() -> PathBuf {
        Constants::root_dir().join("data")
    }
}
