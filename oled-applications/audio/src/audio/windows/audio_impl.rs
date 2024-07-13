use std::sync::{Arc, Mutex};
use tokio::runtime::Handle;
use tokio::sync::mpsc::Sender;
use windows::Win32::Media::Audio::{
    IMMDeviceEnumerator, IMMNotificationClient, MMDeviceEnumerator,
};
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};

use crate::audio::windows::endpoint_volume::EndpointVolume;
use crate::audio::windows::notification_client::NotificationClient;
use crate::AudioData;

pub struct AudioImpl {
    _endpoint_volume: Arc<Mutex<EndpointVolume>>,
    _enumerator: IMMDeviceEnumerator,
    _notification_client: IMMNotificationClient,
}

impl AudioImpl {
    pub fn new(tx: Sender<AudioData>, handle: Handle) -> Self {
        let endpoint_volume = Arc::new(Mutex::new(EndpointVolume::new(tx.clone(), handle.clone())));

        let notification_client = NotificationClient::new({
            let endpoint_volume = Arc::clone(&endpoint_volume);
            move |_device_id| {
                *endpoint_volume.lock().unwrap() = EndpointVolume::new(tx.clone(), handle.clone());
            }
        });

        let enumerator: IMMDeviceEnumerator =
            unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_INPROC_SERVER).unwrap() };
        unsafe {
            enumerator
                .RegisterEndpointNotificationCallback(&notification_client)
                .unwrap();
        }

        Self {
            _endpoint_volume: endpoint_volume,
            _enumerator: enumerator,
            _notification_client: notification_client,
        }
    }
}
