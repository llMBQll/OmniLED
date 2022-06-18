use windows::core::Interface;
use windows::Win32::{
    Media::Audio::{
        eConsole, Endpoints::IAudioEndpointVolume, eRender, IMMDevice,
        IMMDeviceEnumerator, MMDeviceEnumerator
    },
    System::Com::{
        CLSCTX_INPROC_SERVER, CoCreateInstance,
    },
};
use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
use windows::Win32::System::Com::StructuredStorage::STGM_READ;
use windows::Win32::UI::Shell::PropertiesSystem::PropVariantToStringAlloc;

use crate::winapi::pwstr_to_string;

pub struct Audio {
    pub name: String,
    pub volume: i32,
    pub is_muted: bool,
    endpoint_volume: IAudioEndpointVolume,
}

impl Audio {
    pub fn new() -> windows::core::Result<Self> {
        unsafe {
            let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_INPROC_SERVER)?;
            let device: IMMDevice = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;

            let mut endpoint_volume: Option<IAudioEndpointVolume> = None;
            device.Activate(
                &IAudioEndpointVolume::IID,
                CLSCTX_INPROC_SERVER,
                std::ptr::null_mut(),
                &mut endpoint_volume as *mut _ as _,
            )?;
            let endpoint_volume = endpoint_volume.unwrap();

            let props = device.OpenPropertyStore(STGM_READ)?;
            let prop = props.GetValue(&PKEY_Device_FriendlyName)?;
            let name = PropVariantToStringAlloc(&prop)?;

            let name = pwstr_to_string(&name);
            let volume = endpoint_volume.GetMasterVolumeLevelScalar()?;
            let volume = (volume * 100.0).round() as i32;
            let is_muted = endpoint_volume.GetMute()?.as_bool();

            Ok(Self {
                name,
                volume,
                is_muted,
                endpoint_volume,
            })
        }
    }

    pub fn update(&mut self) -> windows::core::Result<bool> {
        unsafe {
            let volume = self.endpoint_volume.GetMasterVolumeLevelScalar()?;
            let volume = (volume * 100.0).round() as i32;
            let is_muted = self.endpoint_volume.GetMute()?.as_bool();

            let res = volume != self.volume || is_muted != self.is_muted;
            if res {
                self.volume = volume;
                self.is_muted = is_muted;
            }
            Ok(res)
        }
    }
}