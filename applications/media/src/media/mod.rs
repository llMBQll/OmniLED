use std::sync::{Arc, Mutex};

use crate::media::session_data::SessionData;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "windows")]
type MediaImpl = windows::media_impl::MediaImpl;

#[cfg(target_os = "linux")]
type MediaImpl = linux::media_impl::MediaImpl;

pub mod session_data;

pub type Callback = dyn FnMut(&String, &SessionData, bool) + Send;

pub struct Media {
    inner: MediaImpl,
}

impl Media {
    pub fn new(callback: Arc<Mutex<Callback>>) -> Self {
        Self {
            inner: MediaImpl::new(callback),
        }
    }

    pub async fn run(&self) {
        self.inner.run().await
    }
}
