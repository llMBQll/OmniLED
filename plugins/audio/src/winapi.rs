use std::ptr::slice_from_raw_parts;

use windows::core::PWSTR;
use windows::Win32::System::Com::{CoInitialize, CoUninitialize};

pub fn initialize() -> windows::core::Result<()> {
    unsafe { CoInitialize(std::ptr::null()) }
}

pub fn finalize() {
    unsafe { CoUninitialize() }
}

pub fn pwstr_len(str: &PWSTR) -> usize {
    if str.is_null() {
        return 0;
    }

    let mut buf = str.0;
    let mut len: usize = 0;
    unsafe {
        while *buf != 0 {
            buf = buf.add(1);
            len += 1;
        }
    }
    len
}

pub fn pwstr_to_string(str: &PWSTR) -> String {
    let len = pwstr_len(str);
    if len == 0 {
        return String::new();
    }
    unsafe {
        let slice = slice_from_raw_parts(str.0, len);
        String::from_utf16_lossy(&*slice)
    }
}
