#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "windows")]
pub type Audio = windows::audio_impl::AudioImpl;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
pub type Audio = linux::audio_impl::AudioImpl;
