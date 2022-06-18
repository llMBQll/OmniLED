use std::ffi::c_void;

pub use crate::managed_string::ManagedString;

#[repr(i32)]
#[derive(Debug)]
pub enum StatusCode {
    Ok = 0,
    Error = 1,
}

pub type Context = c_void;
pub type InitializeFn = fn(*mut *mut Context) -> StatusCode;
pub type NameFn = fn(*mut Context, *mut ManagedString) -> StatusCode;
pub type TypesFn = fn(*mut Context, *mut ManagedString) -> StatusCode;
pub type UpdateFn = fn(*mut Context, *mut ManagedString) -> StatusCode;
pub type FinalizeFn = fn(*mut Context) -> StatusCode;