use log::warn;
use tokio::runtime::Handle;
use tokio::sync::mpsc::Sender;

use crate::{DeviceData, DeviceType};

pub struct AudioImpl;

impl AudioImpl {
    pub fn new(_tx: Sender<(DeviceData, DeviceType)>, _handle: Handle) -> Self {
        _ = DeviceData::new(false, false, 0, None);
        warn!("Application 'audio' is not implemented on macOS");
        Self
    }
}
