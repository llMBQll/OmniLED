use windows::Win32::System::Com::{CoInitialize, CoUninitialize};

pub struct ComGuard;

impl ComGuard {
    pub fn new() -> Self {
        unsafe {
            CoInitialize(None).unwrap();
        }
        Self {}
    }
}

impl Drop for ComGuard {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}
