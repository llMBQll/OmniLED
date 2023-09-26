#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "windows")]
type AudioImpl = windows::audio_impl::AudioImpl;

#[cfg(target_os = "linux")]
type AudioImpl = linux::audio_impl::AudioImpl;

pub struct Audio {
    _inner: AudioImpl,
}

impl Audio {
    pub fn new(volume_callback: fn(bool, i32, Option<String>)) -> Self {
        Self {
            _inner: AudioImpl::new(volume_callback),
        }
    }
}
