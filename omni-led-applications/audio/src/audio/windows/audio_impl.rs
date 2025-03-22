/*
 * OmniLED is a software for displaying data on various OLED devices.
 * Copyright (C) 2024  Michał Bałabanow <m.balabanow@gmail.com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::sync::{Arc, Mutex};
use tokio::runtime::Handle;
use tokio::sync::mpsc::Sender;
use windows::Win32::Media::Audio::{
    IMMDeviceEnumerator, IMMNotificationClient, MMDeviceEnumerator,
};
use windows::Win32::System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance};

use crate::audio::windows::endpoint_volume::EndpointVolume;
use crate::audio::windows::notification_client::NotificationClient;
use crate::{DeviceData, DeviceType};

pub struct AudioImpl {
    _input_endpoint_volume: Arc<Mutex<EndpointVolume>>,
    _output_endpoint_volume: Arc<Mutex<EndpointVolume>>,
    _enumerator: IMMDeviceEnumerator,
    _notification_client: IMMNotificationClient,
}

impl AudioImpl {
    pub fn new(tx: Sender<(DeviceData, DeviceType)>, handle: Handle) -> Self {
        let input_endpoint_volume = Arc::new(Mutex::new(EndpointVolume::new(
            tx.clone(),
            handle.clone(),
            DeviceType::Input,
        )));
        let output_endpoint_volume = Arc::new(Mutex::new(EndpointVolume::new(
            tx.clone(),
            handle.clone(),
            DeviceType::Output,
        )));

        let notification_client = NotificationClient::new({
            let input_endpoint_volume = Arc::clone(&input_endpoint_volume);
            let output_endpoint_volume = Arc::clone(&output_endpoint_volume);
            move |device_type| match device_type {
                DeviceType::Input => {
                    *input_endpoint_volume.lock().unwrap() =
                        EndpointVolume::new(tx.clone(), handle.clone(), DeviceType::Input);
                }
                DeviceType::Output => {
                    *output_endpoint_volume.lock().unwrap() =
                        EndpointVolume::new(tx.clone(), handle.clone(), DeviceType::Output);
                }
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
            _input_endpoint_volume: input_endpoint_volume,
            _output_endpoint_volume: output_endpoint_volume,
            _enumerator: enumerator,
            _notification_client: notification_client,
        }
    }
}
