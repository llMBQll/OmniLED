use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use tokio::runtime::Handle;
use windows::Win32::Media::Audio::{
    IMMDeviceEnumerator, IMMNotificationClient, MMDeviceEnumerator,
};
use windows::Win32::System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance};

use crate::audio::windows::com_guard::ComGuard;
use crate::audio::windows::endpoint_volume::EndpointVolume;
use crate::audio::windows::notification_client::NotificationClient;
use crate::{DeviceData, DeviceType};

pub struct AudioImpl {
    _com_guard: ComGuard,
    _enumerator: IMMDeviceEnumerator,
    _notification_client: IMMNotificationClient,
}

impl AudioImpl {
    pub fn new(tx: tokio::sync::mpsc::Sender<(DeviceData, DeviceType)>, handle: Handle) -> Self {
        let (endpoint_thread_tx, endpoint_thread_rx): (Sender<DeviceType>, Receiver<DeviceType>) =
            mpsc::channel();

        std::thread::spawn(move || {
            let _com_guard = ComGuard::new();

            let mut _input_endpoint_volume =
                EndpointVolume::new(tx.clone(), handle.clone(), DeviceType::Input);
            let mut _output_endpoint_volume =
                EndpointVolume::new(tx.clone(), handle.clone(), DeviceType::Output);

            while let Ok(device_type) = endpoint_thread_rx.recv() {
                match device_type {
                    DeviceType::Input => {
                        _input_endpoint_volume =
                            EndpointVolume::new(tx.clone(), handle.clone(), DeviceType::Input);
                    }
                    DeviceType::Output => {
                        _output_endpoint_volume =
                            EndpointVolume::new(tx.clone(), handle.clone(), DeviceType::Output);
                    }
                }
            }
        });

        let com_guard = ComGuard::new();

        let notification_client = NotificationClient::new(move |device_type| {
            endpoint_thread_tx.send(device_type).unwrap();
        });

        let enumerator: IMMDeviceEnumerator =
            unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_INPROC_SERVER).unwrap() };
        unsafe {
            enumerator
                .RegisterEndpointNotificationCallback(&notification_client)
                .unwrap();
        }

        Self {
            _com_guard: com_guard,
            _enumerator: enumerator,
            _notification_client: notification_client,
        }
    }
}
