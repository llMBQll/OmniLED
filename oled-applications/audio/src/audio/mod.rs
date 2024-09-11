#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "windows")]
type AudioImpl = windows::audio_impl::AudioImpl;

#[cfg(target_os = "linux")]
type AudioImpl = linux::audio_impl::AudioImpl;

use tokio::runtime::Handle;
use tokio::sync::mpsc::Sender;

use crate::AudioData;

pub struct Audio {
    _inner: AudioImpl,
}

impl Audio {
    pub fn new(tx: Sender<AudioData>, handle: Handle) -> Self {
        Self {
            _inner: AudioImpl::new(tx, handle),
        }
    }
}
