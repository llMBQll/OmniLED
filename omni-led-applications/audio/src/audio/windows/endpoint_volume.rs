use tokio::runtime::Handle;
use tokio::sync::mpsc::Sender;
use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
use windows::Win32::Media::Audio::Endpoints::{IAudioEndpointVolume, IAudioEndpointVolumeCallback};
use windows::Win32::Media::Audio::{
    EDataFlow, IMMDeviceEnumerator, MMDeviceEnumerator, eCapture, eConsole, eRender,
};
use windows::Win32::System::Com::StructuredStorage::PropVariantToStringAlloc;
use windows::Win32::System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance, STGM_READ};

use crate::audio::windows::audio_endpoint_volume_callback::AudioEndpointVolumeCallback;
use crate::{DeviceData, DeviceType};

pub struct EndpointVolume {
    endpoint_volume: IAudioEndpointVolume,
    endpoint_volume_callback: IAudioEndpointVolumeCallback,
}

impl EndpointVolume {
    pub fn new(
        tx: Sender<(DeviceData, DeviceType)>,
        handle: Handle,
        device_type: DeviceType,
    ) -> Option<Self> {
        let endpoint_volume_callback =
            AudioEndpointVolumeCallback::new(tx.clone(), handle.clone(), device_type);
        let endpoint_volume = unsafe {
            let enumerator: IMMDeviceEnumerator =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_INPROC_SERVER).unwrap();

            match enumerator
                .GetDefaultAudioEndpoint(Self::device_type_to_data_flow(device_type), eConsole)
            {
                Ok(device) => {
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
                        tx.send((DeviceData::new(true, mute, volume, Some(name)), device_type))
                            .await
                            .unwrap()
                    });

                    endpoint_volume
                }
                Err(_err) => {
                    handle.spawn(async move {
                        tx.send((
                            DeviceData::new(false, true, 0, Some("".to_string())),
                            device_type,
                        ))
                        .await
                        .unwrap()
                    });
                    return None;
                }
            }
        };

        Some(Self {
            endpoint_volume,
            endpoint_volume_callback,
        })
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
    }
}
