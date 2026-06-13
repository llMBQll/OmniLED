use log::info;
use mlua::ErrorContext;

#[cfg(target_os = "linux")]
mod autostart_linux;

#[cfg(target_os = "linux")]
pub type AutoStart = autostart_linux::AutoStart;

#[cfg(target_os = "macos")]
mod autostart_macos;

#[cfg(target_os = "macos")]
pub type AutoStart = autostart_macos::AutoStart;

#[cfg(target_os = "windows")]
mod autostart_windows;

#[cfg(target_os = "windows")]
pub type AutoStart = autostart_windows::AutoStart;

pub trait AutoStartInterface {
    fn enable() -> mlua::Result<()>;

    fn disable() -> mlua::Result<()>;

    fn enabled() -> mlua::Result<bool>;

    fn toggle() -> mlua::Result<bool> {
        let enabled = Self::enabled()?;
        if enabled {
            Self::disable().with_context(|_| "When trying to disable auto start")?;
            info!("Autostart disabled successfully")
        } else {
            Self::enable().with_context(|_| "When trying to enable auto start")?;
            info!("Autostart enabled successfully");
        }
        return Ok(!enabled);
    }
}
