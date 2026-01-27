pub mod session_data;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "windows")]
pub type Media = windows::media_impl::MediaImpl;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
pub type Media = linux::media_impl::MediaImpl;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "macos")]
pub type Media = macos::media_impl::MediaImpl;
