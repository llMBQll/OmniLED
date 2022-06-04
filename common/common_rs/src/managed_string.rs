use std::alloc::{alloc, dealloc, Layout};
use std::ffi::CStr;
use std::fmt::{Display, Formatter};
use std::os::raw::c_char;
use std::ptr::{copy_nonoverlapping, null_mut};

#[repr(C)]
pub struct ManagedString {
    str: *mut u8,
    len: usize,
    del: fn(*mut u8, usize),
}

impl From<&String> for ManagedString {
    fn from(string: &String) -> Self {
        let len = string.len();
        let layout = Layout::from_size_align(len + 1, 1).expect("Invalid layout");
        let str = unsafe {
            let str = alloc(layout);
            copy_nonoverlapping(string.as_ptr(), str, len);
            *str.add(len) = 0;
            str
        };

        Self {
            str,
            len,
            del: ManagedString::default_deleter,
        }
    }
}

impl Drop for ManagedString {
    fn drop(&mut self) {
        (self.del)(self.str, self.len);
    }
}

impl ManagedString {
    fn default_deleter(str: *mut u8, len: usize) {
        println!("[Rust] Deleter called on {:p}", str);
        if str.is_null() {
            return;
        }
        let layout = Layout::from_size_align(len + 1, 1).expect("Invalid layout");
        unsafe { dealloc(str as *mut u8, layout); }
    }

    pub fn new() -> Self {
        Self {
            str: null_mut(),
            len: 0,
            del: ManagedString::default_deleter,
        }
    }

    pub fn to_str(&self) -> Result<&str, std::str::Utf8Error> {
        let tmp = unsafe { CStr::from_ptr(self.str as *const c_char) };
        tmp.to_str()
    }
}

impl Display for ManagedString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str().expect("Conversion error"))
    }
}