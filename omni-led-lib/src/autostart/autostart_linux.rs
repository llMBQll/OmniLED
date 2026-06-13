use std::path::PathBuf;

use crate::autostart::AutoStartInterface;

pub struct AutoStart;

impl AutoStart {
    fn desktop_file_path() -> Option<PathBuf> {
        let autostart_dir = if let Ok(xdg_config_home) = std::env::var("XDG_CONFIG_HOME") {
            Some(PathBuf::from(xdg_config_home).join("autostart"))
        } else if let Ok(home) = std::env::var("HOME") {
            Some(PathBuf::from(home).join(".config").join("autostart"))
        } else {
            None
        };

        autostart_dir.map(|autostart_dir| autostart_dir.join("OmniLED").with_extension(".desktop"))
    }

    fn generate_desktop_file_content() -> mlua::Result<String> {
        let exe_path = std::env::current_exe().map_err(mlua::Error::external)?;

        // TODO: add icon entry, needs asset rework first

        let mut content = String::new();
        content += "[Desktop Entry]\n";
        content += &format!("Exec={}\n", exe_path.display());
        content += "Name=OmniLED\n";
        content += "Terminal=False\n";
        content += "Type=Application\n";
        Ok(content)
    }
}

impl AutoStartInterface for AutoStart {
    fn enable() -> mlua::Result<()> {
        if let Some(path) = Self::desktop_file_path() {
            let content = Self::generate_desktop_file_content()?;
            std::fs::write(path, content).map_err(mlua::Error::external)
        } else {
            Err(mlua::Error::runtime(
                "Failed to get the autostart directory",
            ))
        }
    }

    fn disable() -> mlua::Result<()> {
        if let Some(path) = Self::desktop_file_path() {
            std::fs::remove_file(path).map_err(mlua::Error::external)
        } else {
            Err(mlua::Error::runtime(
                "Failed to get the autostart directory",
            ))
        }
    }

    fn enabled() -> mlua::Result<bool> {
        if let Some(path) = Self::desktop_file_path() {
            std::fs::exists(path).map_err(mlua::Error::external)
        } else {
            Err(mlua::Error::runtime(
                "Failed to get the autostart directory",
            ))
        }
    }
}
