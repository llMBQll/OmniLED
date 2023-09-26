use std::sync::{Arc, Mutex};
use windows::Win32::Media::Audio::{
    IMMDeviceEnumerator, IMMNotificationClient, MMDeviceEnumerator,
};
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};

use crate::audio::windows::endpoint_volume::EndpointVolume;
use crate::audio::windows::notification_client::NotificationClient;

pub struct AudioImpl {
    _endpoint_volume: Arc<Mutex<EndpointVolume>>,
    _enumerator: IMMDeviceEnumerator,
    _notification_client: IMMNotificationClient,
}

impl AudioImpl {
    pub fn new(volume_callback: fn(bool, i32, Option<String>)) -> Self {
        let endpoint_volume = Arc::new(Mutex::new(EndpointVolume::new(volume_callback.clone())));

        let notification_client = NotificationClient::new({
            let endpoint_volume = Arc::clone(&endpoint_volume);
            move |_device_id| {
                *endpoint_volume.lock().unwrap() = EndpointVolume::new(volume_callback.clone());
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
