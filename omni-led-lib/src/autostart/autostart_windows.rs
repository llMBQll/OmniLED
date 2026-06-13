use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;
use windows::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, CoCreateInstance, CoInitialize, CoTaskMemFree, CoUninitialize,
    IPersistFile,
};
use windows::Win32::UI::Shell::{
    FOLDERID_Startup, IShellLinkW, KNOWN_FOLDER_FLAG, SHGetKnownFolderPath, ShellLink,
};
use windows::core::{GUID, Interface, PCWSTR};

use crate::autostart::AutoStartInterface;
use crate::constants::constants::Constants;
use crate::defer;

pub struct AutoStart;

impl AutoStart {
    fn autostart_file_path() -> mlua::Result<PathBuf> {
        unsafe {
            let autostart_dir_pwstr =
                SHGetKnownFolderPath(&FOLDERID_Startup as *const GUID, KNOWN_FOLDER_FLAG(0), None)
                    .map_err(mlua::Error::external)?;
            defer!({ CoTaskMemFree(Some(autostart_dir_pwstr.0 as *const _)) });

            autostart_dir_pwstr
                .to_string()
                .map(|autostart_dir| {
                    PathBuf::from(autostart_dir)
                        .join("OmniLED")
                        .with_extension("lnk")
                })
                .map_err(mlua::Error::external)
        }
    }

    fn create_link(link_path: &PathBuf) -> mlua::Result<()> {
        fn to_wide_string(s: &OsStr) -> Vec<u16> {
            s.encode_wide().chain(std::iter::once(0u16)).collect()
        }

        let exe_path = Constants::current_exe();

        unsafe {
            CoInitialize(None).unwrap();
            defer!({ CoUninitialize() });

            let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)
                .map_err(mlua::Error::external)?;

            let exe_path = to_wide_string(exe_path.as_os_str());
            link.SetPath(PCWSTR::from_raw(exe_path.as_ptr()))
                .map_err(mlua::Error::external)?;

            let link_path = to_wide_string(link_path.as_os_str());
            let persist: IPersistFile = link.cast().map_err(mlua::Error::external)?;
            persist
                .Save(PCWSTR::from_raw(link_path.as_ptr()), true)
                .map_err(mlua::Error::external)?;
        }

        Ok(())
    }
}

impl AutoStartInterface for AutoStart {
    fn enable() -> mlua::Result<()> {
        let path = Self::autostart_file_path()?;
        Self::create_link(&path)?;
        Ok(())
    }

    fn disable() -> mlua::Result<()> {
        let path = Self::autostart_file_path()?;
        std::fs::remove_file(path).map_err(mlua::Error::external)
    }

    fn enabled() -> mlua::Result<bool> {
        let path = Self::autostart_file_path()?;
        std::fs::exists(path).map_err(mlua::Error::external)
    }
}
