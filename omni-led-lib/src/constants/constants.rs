use mlua::{Lua, UserData, UserDataFields};
use std::path::PathBuf;
use std::{
    env::consts::{EXE_EXTENSION, EXE_SUFFIX, OS},
    path::MAIN_SEPARATOR_STR,
};

use crate::common::user_data::{UniqueUserData, UserDataRef};

#[derive(Debug)]
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

const ENV_CONFIG_DIR: &str = "OMNILED_CONFIG_DIR";

impl Constants {
    pub fn load(lua: &Lua, config_path_override: Option<PathBuf>) {
        Self::set_unique(
            lua,
            Self {
                applications_dir: Self::exe_dir(),
                config_dir: Self::resolve_config_dir(config_path_override),
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

    fn resolve_config_dir(root_override: Option<PathBuf>) -> PathBuf {
        root_override.unwrap_or_else(|| match std::env::var(ENV_CONFIG_DIR) {
            Ok(env_config_path) => PathBuf::from(env_config_path),
            Err(_) => Self::root_dir().join("config"),
        })
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    // Sync tests that access set and get the `ENV_CONFIG_DIR` variable so that they don't
    // interfere with each other.
    static ENV_TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn env_test_lock() -> &'static Mutex<()> {
        ENV_TEST_LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn resolve_config_dir_with_path_override() {
        let _guard = env_test_lock().lock().unwrap();

        unsafe {
            std::env::set_var(ENV_CONFIG_DIR, "[Env] OmniLED");
        }

        let root_dir = Constants::resolve_config_dir(Some(PathBuf::from("[CLI] OmniLED")));
        assert_eq!(root_dir, PathBuf::from("[CLI] OmniLED"));
    }

    #[test]
    fn resolve_config_dir_with_env_override() {
        let _guard = env_test_lock().lock().unwrap();

        unsafe {
            std::env::set_var(ENV_CONFIG_DIR, "[Env] OmniLED");
        }

        let root_dir = Constants::resolve_config_dir(None);
        assert_eq!(root_dir, PathBuf::from("[Env] OmniLED"));
    }

    #[test]
    fn resolve_config_dir_with_no_override() {
        let _guard = env_test_lock().lock().unwrap();

        unsafe {
            std::env::remove_var(ENV_CONFIG_DIR);
        }

        let root_dir = Constants::resolve_config_dir(None);
        assert_eq!(root_dir, Constants::root_dir().join("config"));
    }
}
