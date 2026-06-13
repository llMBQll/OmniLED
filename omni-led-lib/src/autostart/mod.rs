#[cfg(target_os = "linux")]
mod autostart_linux;

#[cfg(target_os = "linux")]
pub type AutoStart = autostart_linux::AutoStart;

trait AutoStartInterface {
    fn enable() -> mlua::Result<()>;

    fn disable() -> mlua::Result<()>;

    fn enabled() -> mlua::Result<bool>;

    fn toggle() -> mlua::Result<()> {
        if Self::enabled()? {
            Self::disable()
        } else {
            Self::enable()
        }
    }
}
