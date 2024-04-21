use mlua::{chunk, Lua};
use std::path::PathBuf;
use std::{
    env::consts::{
        ARCH, DLL_EXTENSION, DLL_PREFIX, DLL_SUFFIX, EXE_EXTENSION, EXE_SUFFIX, FAMILY, OS,
    },
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
            Arch = $ARCH,
            DllExtension = $DLL_EXTENSION,
            DllPrefix = $DLL_PREFIX,
            DllSuffix = $DLL_SUFFIX,
            ExeExtension = $EXE_EXTENSION,
            ExeSuffix = $EXE_SUFFIX,
            Family = $FAMILY,
            PathSeparator = $MAIN_SEPARATOR_STR,
            Os = $OS,
        });
        lua.globals().set("PLATFORM", platform).unwrap();
    }

    #[cfg(debug_assertions)]
    pub fn root_dir() -> PathBuf {
        let root_dir = PathBuf::from(".");
        root_dir
    }

    #[cfg(not(debug_assertions))]
    pub fn root_dir() -> PathBuf {
        let root_dir = dirs_next::config_dir().expect("Couldn't get default config directory");
        let root_dir = root_dir.join("SteelseriesOLED");
        root_dir
    }

    pub fn config_dir() -> PathBuf {
        Self::root_dir().join("config")
    }

    #[cfg(debug_assertions)]
    pub fn applications_dir() -> PathBuf {
        Self::root_dir().join("target").join("debug")
    }

    #[cfg(not(debug_assertions))]
    pub fn applications_dir() -> PathBuf {
        Self::root_dir().join("applications")
    }
}
