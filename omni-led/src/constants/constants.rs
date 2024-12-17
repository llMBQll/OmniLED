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
