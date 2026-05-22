use log::warn;
use tokio::sync::mpsc::Sender;

use crate::Data;

pub struct MediaImpl;

impl MediaImpl {
    pub async fn run(_tx: Sender<Data>) {
        warn!("Application 'media' is not implemented on macOS");
    }
}
