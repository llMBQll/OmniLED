use crate::Message;
use tokio::sync::mpsc::Sender;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "windows")]
type MediaImpl = windows::media_impl::MediaImpl;

#[cfg(target_os = "linux")]
type MediaImpl = linux::media_impl::MediaImpl;

pub mod session_data;

pub struct Media {
    inner: MediaImpl,
}

impl Media {
    pub fn new(tx: Sender<Message>) -> Self {
        Self {
            inner: MediaImpl::new(tx),
        }
    }

    pub async fn run(&self) {
        self.inner.run().await
    }
}
