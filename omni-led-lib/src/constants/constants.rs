use mlua::{Lua, UserData, UserDataFields};
use std::path::PathBuf;
use std::{
    env::consts::{DLL_EXTENSION, DLL_PREFIX, DLL_SUFFIX, EXE_EXTENSION, EXE_SUFFIX, OS},
    path::MAIN_SEPARATOR_STR,
};

use crate::common::lua_traits::LuaName;
use crate::common::user_data::{UserDataRef, set_unique_user_data};

#[derive(Debug, Clone)]
pub struct Constants {
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub dll_extension: &'static str,
    pub dll_prefix: &'static str,
    pub dll_suffix: &'static str,
    pub exe_extension: &'static str,
    pub exe_suffix: &'static str,
    pub os: &'static str,
    pub path_separator: &'static str,
    pub plugins_dir: PathBuf,
}

impl Constants {
    pub fn load(lua: &Lua) {
        set_unique_user_data(
            lua,
            Self {
                config_dir: Self::config_dir(),
                data_dir: Self::data_dir(),
                dll_extension: DLL_EXTENSION,
                dll_prefix: DLL_PREFIX,
                dll_suffix: DLL_SUFFIX,
                exe_extension: EXE_EXTENSION,
                exe_suffix: EXE_SUFFIX,
                os: OS,
                path_separator: MAIN_SEPARATOR_STR,
                plugins_dir: Self::exe_dir(),
            },
        );

        // Logging isn't initialized yet...
        std::println!("{:#?}", UserDataRef::<Self>::load(lua).get());
    }

    pub fn config_dir() -> PathBuf {
        #[cfg(feature = "dev")]
        let dir = Self::root_dir().join("config");

        #[cfg(not(feature = "dev"))]
        let dir = Self::system_config_dir().join("config");

        dir
    }

    pub fn current_exe() -> PathBuf {
        std::env::current_exe().unwrap()
    }

    pub fn data_dir() -> PathBuf {
        #[cfg(feature = "dev")]
        let dir = Self::root_dir().join("data");

        #[cfg(not(feature = "dev"))]
        let dir = Self::system_config_dir().join("data");

        dir
    }

    pub fn exe_dir() -> PathBuf {
        Self::current_exe().parent().unwrap().to_path_buf()
    }

    pub fn license_path() -> PathBuf {
        #[cfg(feature = "dev")]
        let path = Self::root_dir().join("LICENSE");

        #[cfg(not(feature = "dev"))]
        let path = Self::resource_dir().join("license");

        path
    }

    pub fn plugins_dir() -> PathBuf {
        #[cfg(feature = "dev")]
        let path = Self::exe_dir();

        #[cfg(not(feature = "dev"))]
        let path = Self::resource_dir().join("plugins");

        path
    }

    pub fn system_config_dir() -> PathBuf {
        dirs_next::config_dir().unwrap().join("OmniLED")
    }

    #[cfg(not(feature = "dev"))]
    fn resource_dir() -> PathBuf {
        #[cfg(target_os = "linux")]
        let path = Self::exe_dir().parent().unwrap().join("lib/omni-led");

        #[cfg(target_os = "macos")]
        let path = Self::exe_dir().parent().unwrap().join("Resources");

        #[cfg(target_os = "windows")]
        let path = Self::exe_dir();

        path
    }

    #[cfg(feature = "dev")]
    fn root_dir() -> PathBuf {
        Self::exe_dir()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf()
    }
}

impl LuaName for Constants {
    const NAME: &str = "PLATFORM";
}

impl UserData for Constants {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("ConfigDir", |_, constants| {
            Ok(constants.config_dir.to_str().unwrap().to_string())
        });
        fields.add_field_method_get("DllExtension", |_, constants| Ok(constants.dll_extension));
        fields.add_field_method_get("DllPrefix", |_, constants| Ok(constants.dll_prefix));
        fields.add_field_method_get("DllSuffix", |_, constants| Ok(constants.dll_suffix));
        fields.add_field_method_get("ExeExtension", |_, constants| Ok(constants.exe_extension));
        fields.add_field_method_get("ExeSuffix", |_, constants| Ok(constants.exe_suffix));
        fields.add_field_method_get("Os", |_, constants| Ok(constants.os));
        fields.add_field_method_get("PathSeparator", |_, constants| Ok(constants.path_separator));
        fields.add_field_method_get("PluginsDir", |_, constants| {
            Ok(constants.plugins_dir.to_str().unwrap().to_string())
        });
    }
}
