use std::{
    env::consts::{
        ARCH, DLL_EXTENSION, DLL_PREFIX, DLL_SUFFIX, EXE_EXTENSION, EXE_SUFFIX, FAMILY, OS,
    },
    path::MAIN_SEPARATOR_STR,
};

use mlua::{chunk, Lua};

pub struct Constants;

impl Constants {
    pub fn load(lua: &Lua) {
        lua.load(chunk! {
            PLATFORM = {
                Arch = $ARCH,
                DllExtension = $DLL_EXTENSION,
                DllPrefix = $DLL_PREFIX,
                DllSuffix = $DLL_SUFFIX,
                ExeExtension = $EXE_EXTENSION,
                ExeSuffix = $EXE_SUFFIX,
                Family = $FAMILY,
                PathSeparator = $MAIN_SEPARATOR_STR,
                Os = $OS,
            }
        })
        .exec()
        .unwrap();
    }
}
