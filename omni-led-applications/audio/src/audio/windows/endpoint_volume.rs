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

use tokio::runtime::Handle;
use tokio::sync::mpsc::Sender;
use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
use windows::Win32::Media::Audio::Endpoints::{
    IAudioEndpointVolume, IAudioEndpointVolume_Impl, IAudioEndpointVolumeCallback,
};
use windows::Win32::Media::Audio::{
    EDataFlow, IMMDevice, IMMDeviceEnumerator, MMDeviceEnumerator, eCapture, eConsole, eRender,
};
use windows::Win32::System::Com::StructuredStorage::PropVariantToStringAlloc;
use windows::Win32::System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance, STGM_READ};
use windows::core::{BOOL, GUID, Ref, implement};

use crate::audio::windows::audio_endpoint_volume_callback::AudioEndpointVolumeCallback;
use crate::audio::windows::com_guard::ComGuard;
use crate::{DeviceData, DeviceType};

pub struct EndpointVolume {
    _com_guard: ComGuard,
    endpoint_volume: IAudioEndpointVolume,
    endpoint_volume_callback: IAudioEndpointVolumeCallback,
}

impl EndpointVolume {
    pub fn new(
        tx: Sender<(DeviceData, DeviceType)>,
        handle: Handle,
        device_type: DeviceType,
    ) -> Self {
        let com_guard = ComGuard::new();
        let endpoint_volume_callback =
            AudioEndpointVolumeCallback::new(tx.clone(), handle.clone(), device_type);
        let endpoint_volume = unsafe {
            let enumerator: IMMDeviceEnumerator =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_INPROC_SERVER).unwrap();
            let device: IMMDevice = enumerator
                .GetDefaultAudioEndpoint(Self::device_type_to_data_flow(device_type), eConsole)
                .unwrap();
            let endpoint_volume: IAudioEndpointVolume =
                device.Activate(CLSCTX_INPROC_SERVER, None).unwrap();
            endpoint_volume
                .RegisterControlChangeNotify(&endpoint_volume_callback)
                .unwrap();

            let mute = endpoint_volume.GetMute().unwrap().into();

            let volume = endpoint_volume.GetMasterVolumeLevelScalar().unwrap();
            let volume = (volume * 100.0).round() as i32;

            let props = device.OpenPropertyStore(STGM_READ).unwrap();
            let prop = props.GetValue(&PKEY_Device_FriendlyName).unwrap();
            let name = PropVariantToStringAlloc(&prop).unwrap();
            let name = name.to_string().unwrap();

            handle.spawn(async move {
                tx.send((DeviceData::new(mute, volume, Some(name)), device_type))
                    .await
                    .unwrap()
            });

            endpoint_volume
        };

        Self {
            _com_guard: com_guard,
            endpoint_volume,
            endpoint_volume_callback,
        }
    }

    fn device_type_to_data_flow(device_type: DeviceType) -> EDataFlow {
        match device_type {
            DeviceType::Input => eCapture,
            DeviceType::Output => eRender,
        }
    }
}

impl Drop for EndpointVolume {
    fn drop(&mut self) {
        unsafe {
            self.endpoint_volume
                .UnregisterControlChangeNotify(&self.endpoint_volume_callback)
                .unwrap();
        }

        // This is a temporary solution while I investigate why dropping the IAudioEndpointVolume
        // object hangs the program execution
        let mut endpoint_volume = EmptyAudioEndpointVolume::new();
        std::mem::swap(&mut self.endpoint_volume, &mut endpoint_volume);
        std::mem::forget(endpoint_volume);
    }
}

// This implementation is only needed to create a temporary object that can be dropped without
// hanging the program execution
#[implement(IAudioEndpointVolume)]
struct EmptyAudioEndpointVolume {}

impl EmptyAudioEndpointVolume {
    pub fn new() -> IAudioEndpointVolume {
        let this = Self {};
        this.into()
    }
}

#[allow(non_snake_case)]
impl IAudioEndpointVolume_Impl for EmptyAudioEndpointVolume_Impl {
    fn RegisterControlChangeNotify(
        &self,
        _: Ref<'_, IAudioEndpointVolumeCallback>,
    ) -> windows::core::Result<()> {
        Ok(())
    }

    fn UnregisterControlChangeNotify(
        &self,
        _: Ref<'_, IAudioEndpointVolumeCallback>,
    ) -> windows::core::Result<()> {
        Ok(())
    }

    fn GetChannelCount(&self) -> windows::core::Result<u32> {
        Ok(0)
    }

    fn SetMasterVolumeLevel(&self, _: f32, _: *const GUID) -> windows::core::Result<()> {
        Ok(())
    }

    fn SetMasterVolumeLevelScalar(&self, _: f32, _: *const GUID) -> windows::core::Result<()> {
        Ok(())
    }

    fn GetMasterVolumeLevel(&self) -> windows::core::Result<f32> {
        Ok(0.0)
    }

    fn GetMasterVolumeLevelScalar(&self) -> windows::core::Result<f32> {
        Ok(0.0)
    }

    fn SetChannelVolumeLevel(&self, _: u32, _: f32, _: *const GUID) -> windows::core::Result<()> {
        Ok(())
    }

    fn SetChannelVolumeLevelScalar(
        &self,
        _: u32,
        _: f32,
        _: *const GUID,
    ) -> windows::core::Result<()> {
        Ok(())
    }

    fn GetChannelVolumeLevel(&self, _: u32) -> windows::core::Result<f32> {
        Ok(0.0)
    }

    fn GetChannelVolumeLevelScalar(&self, _: u32) -> windows::core::Result<f32> {
        Ok(0.0)
    }

    fn SetMute(&self, _: BOOL, _: *const GUID) -> windows::core::Result<()> {
        Ok(())
    }

    fn GetMute(&self) -> windows::core::Result<BOOL> {
        Ok(false.into())
    }

    fn GetVolumeStepInfo(&self, _: *mut u32, _: *mut u32) -> windows::core::Result<()> {
        Ok(())
    }

    fn VolumeStepUp(&self, _: *const GUID) -> windows::core::Result<()> {
        Ok(())
    }

    fn VolumeStepDown(&self, _: *const GUID) -> windows::core::Result<()> {
        Ok(())
    }

    fn QueryHardwareSupport(&self) -> windows::core::Result<u32> {
        Ok(0)
    }

    fn GetVolumeRange(&self, _: *mut f32, _: *mut f32, _: *mut f32) -> windows::core::Result<()> {
        Ok(())
    }
}
