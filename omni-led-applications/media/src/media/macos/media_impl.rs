use log::warn;
use tokio::sync::mpsc::Sender;

use crate::Data;

pub struct MediaImpl;

impl MediaImpl {
    pub fn new(_tx: Sender<Data>) -> Self {
        Self
    }

    pub async fn run(&self) {
        warn!("Application 'media' is not implemented on macOS");
    }
}
