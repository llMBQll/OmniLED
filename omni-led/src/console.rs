use windows::Win32::System::Console::{AttachConsole, GetConsoleWindow};

pub fn attach_console_if_missing() {
    if !has_console() {
        const ATTACH_PARENT_PROCESS: u32 = u32::MAX;
        unsafe {
            _ = AttachConsole(ATTACH_PARENT_PROCESS);
        }
    }
}

fn has_console() -> bool {
    unsafe { !GetConsoleWindow().is_invalid() }
}
