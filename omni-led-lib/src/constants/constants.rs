use mlua::{Lua, UserData, UserDataFields};
use std::path::PathBuf;
use std::{
    env::consts::{EXE_EXTENSION, EXE_SUFFIX, OS},
    path::MAIN_SEPARATOR_STR,
};

use crate::common::user_data::{UniqueUserData, UserDataRef};

#[derive(Debug, Clone)]
pub struct Constants {
    pub applications_dir: PathBuf,
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub exe_extension: &'static str,
    pub exe_suffix: &'static str,
    pub os: &'static str,
    pub path_separator: &'static str,
    pub root_dir: PathBuf,
}

impl Constants {
    pub fn load(lua: &Lua) {
        Self::set_unique(
            lua,
            Self {
                applications_dir: Self::exe_dir(),
                config_dir: Self::root_dir().join("config"),
                data_dir: Self::root_dir().join("data"),
                exe_extension: EXE_EXTENSION,
                exe_suffix: EXE_SUFFIX,
                os: OS,
                path_separator: MAIN_SEPARATOR_STR,
                root_dir: Self::root_dir(),
            },
        );

        // Logging isn't initialized yet...
        std::println!("{:#?}", UserDataRef::<Self>::load(lua).get());
    }

    fn root_dir() -> PathBuf {
        #[cfg(feature = "dev")]
        let root = Self::exe_dir()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf();

        #[cfg(not(feature = "dev"))]
        let root = Self::exe_dir().parent().unwrap().to_path_buf();

        root
    }

    fn exe_dir() -> PathBuf {
        std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf()
    }
}

impl UniqueUserData for Constants {
    fn identifier() -> &'static str {
        "PLATFORM"
    }
}

impl UserData for Constants {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("ApplicationsDir", |_, constants| {
            Ok(constants.applications_dir.to_str().unwrap().to_string())
        });
        fields.add_field_method_get("ConfigDir", |_, constants| {
            Ok(constants.config_dir.to_str().unwrap().to_string())
        });
        fields.add_field_method_get("ExeExtension", |_, constants| Ok(constants.exe_extension));
        fields.add_field_method_get("ExeSuffix", |_, constants| Ok(constants.exe_suffix));
        fields.add_field_method_get("Os", |_, constants| Ok(constants.os));
        fields.add_field_method_get("PathSeparator", |_, constants| Ok(constants.path_separator));
        fields.add_field_method_get("RootDir", |_, constants| {
            Ok(constants.root_dir.to_str().unwrap().to_string())
        });
    }
}
