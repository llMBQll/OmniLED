use std::sync::{Arc, Mutex};
use windows::core::*;
use windows::Win32::{
    Media::Audio::{
        Endpoints::{IAudioEndpointVolume, IAudioEndpointVolumeCallback, IAudioEndpointVolumeCallback_Impl},
        eConsole, eRender, IMMDevice, IMMDeviceEnumerator, MMDeviceEnumerator, AUDIO_VOLUME_NOTIFICATION_DATA,
        IMMNotificationClient, IMMNotificationClient_Impl
    },
    System::Com::{
        CLSCTX_INPROC_SERVER, CoCreateInstance,
    },
};
use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
use windows::Win32::Media::Audio::{EDataFlow, ERole};
use windows::Win32::System::Com::StructuredStorage::STGM_READ;
use windows::Win32::UI::Shell::PropertiesSystem::{PropVariantToStringAlloc, PROPERTYKEY};

use crate::winapi::pwstr_to_string;
use common_rs::interface::OnUpdateCallbackFn;

pub struct Audio {
    enumerator: IMMDeviceEnumerator,
    endpoint: Option<IAudioEndpointVolume>,
    volume_callback: Option<IAudioEndpointVolumeCallback>,
    notification_client: Option<IMMNotificationClient>,
    is_muted: bool,
    volume: i32,
    name: String,
    on_update: OnUpdateCallbackFn,
}

impl Audio {
    pub fn new(on_update: OnUpdateCallbackFn) -> Result<Arc<Mutex<Self>>> {
        unsafe {
            let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_INPROC_SERVER)?;
            let device: IMMDevice = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;

            let mut endpoint: Option<IAudioEndpointVolume> = None;
            device.Activate(
                &IAudioEndpointVolume::IID,
                CLSCTX_INPROC_SERVER,
                std::ptr::null_mut(),
                &mut endpoint as *mut _ as _,
            )?;

            let props = device.OpenPropertyStore(STGM_READ)?;
            let prop = props.GetValue(&PKEY_Device_FriendlyName)?;
            let name = PropVariantToStringAlloc(&prop)?;

            let name = pwstr_to_string(&name);

            let audio = Self {
                enumerator,
                endpoint,
                volume_callback: None,
                notification_client: None,
                is_muted: false,
                volume: 0,
                name,
                on_update
            };
            let audio_mutex = Arc::new(Mutex::new(audio));

            let audio = audio_mutex.clone();
            let mut lock = audio.lock();
            let audio = lock.as_mut().unwrap();

            audio.volume_callback = match &audio.endpoint {
                Some(endpoint) => {
                    let volume_callback = AudioEndpointVolumeCallback::new(audio_mutex.clone()).into();
                    endpoint.RegisterControlChangeNotify(&volume_callback).unwrap();
                    Some(volume_callback)
                }
                None => None
            };

            let notification_client = MMNotificationClient::new(audio_mutex.clone()).into();
            audio.enumerator.RegisterEndpointNotificationCallback(&notification_client).unwrap();
            audio.notification_client = Some(notification_client);

            Ok(audio_mutex)
        }
    }

    pub fn send_update(&mut self, is_muted: bool, volume: i32, name: Option<String>) {
        self.is_muted = is_muted;
        self.volume = volume;
        if name.is_some() { self.name = name.unwrap(); }

        let json = format!(r#"{{"Volume":{},"IsMuted":{},"Name":"{}"}}"#, self.volume, self.is_muted, self.name);
        let raw = json.as_bytes();
        (self.on_update)(raw.as_ptr(), raw.len() as u32);
    }

    fn get_default_device_name(&self) -> Result<String> {
        unsafe {
            let device: IMMDevice = self.enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;

            let mut endpoint: Option<IAudioEndpointVolume> = None;
            device.Activate(
                &IAudioEndpointVolume::IID,
                CLSCTX_INPROC_SERVER,
                std::ptr::null_mut(),
                &mut endpoint as *mut _ as _,
            )?;

            let props = device.OpenPropertyStore(STGM_READ)?;
            let prop = props.GetValue(&PKEY_Device_FriendlyName)?;
            let name = PropVariantToStringAlloc(&prop)?;

            let name = pwstr_to_string(&name);
            Ok(name)
        }
    }
}

impl Drop for Audio {
    fn drop(&mut self) {
        if self.endpoint.is_some() && self.volume_callback.is_some() {
            let endpoint = self.endpoint.take().unwrap();
            let volume_callback = self.volume_callback.take().unwrap();
            unsafe {
                endpoint.UnregisterControlChangeNotify(volume_callback).unwrap();
            }
        }
        if self.notification_client.is_some() {
            let notification_client = self.notification_client.take().unwrap();
            unsafe {
                self.enumerator.UnregisterEndpointNotificationCallback(notification_client).unwrap();
            }
        }
    }
}

#[implement(windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolumeCallback)]
struct AudioEndpointVolumeCallback {
    audio: Arc<Mutex<Audio>>
}

impl AudioEndpointVolumeCallback {
    pub fn new(audio: Arc<Mutex<Audio>>) -> Self {
        Self {
            audio
        }
    }
}

#[allow(non_snake_case)]
impl IAudioEndpointVolumeCallback_Impl for AudioEndpointVolumeCallback {
    fn OnNotify(&self, pnotify: *mut AUDIO_VOLUME_NOTIFICATION_DATA) -> Result<()> {
        unsafe {
            let is_muted = (*pnotify).bMuted.as_bool();
            let volume = (*pnotify).fMasterVolume;
            let volume = (volume * 100.0).round() as i32;

            let mut lock = self.audio.lock();
            let audio = lock.as_mut().unwrap();
            audio.send_update(is_muted, volume, None);
        }
        Ok(())
    }
}

#[implement(windows::Win32::Media::Audio::IMMNotificationClient)]
struct MMNotificationClient {
    audio: Arc<Mutex<Audio>>
}

impl MMNotificationClient {
    pub fn new(audio: Arc<Mutex<Audio>>) -> Self {
        Self {
            audio
        }
    }
}

#[allow(non_snake_case)]
impl IMMNotificationClient_Impl for MMNotificationClient {
    fn OnDeviceStateChanged(&self, _device_id: &PCWSTR, _state: u32) -> Result<()> {
        Ok(())
    }

    fn OnDeviceAdded(&self, _device_id: &PCWSTR) -> Result<()> {
        Ok(())
    }

    fn OnDeviceRemoved(&self, _device_id: &PCWSTR) -> Result<()> {
        Ok(())
    }

    fn OnDefaultDeviceChanged(&self, _flow: EDataFlow, _role: ERole, _default_device_id: &PCWSTR) -> Result<()> {
        let mut lock = self.audio.lock();
        let audio = lock.as_mut().unwrap();

        let name = match audio.get_default_device_name() {
            Ok(name) => name,
            Err(_) => String::new()
        };
        let (is_muted, volume) = unsafe {
            let volume = audio.endpoint.as_ref().unwrap().GetMasterVolumeLevelScalar().unwrap();
            let volume = (volume * 100.0).round() as i32;
            let is_muted = audio.endpoint.as_ref().unwrap().GetMute().unwrap().into();
            (is_muted, volume)
        };
        audio.send_update(is_muted, volume, Some(name));
        Ok(())
    }

    fn OnPropertyValueChanged(&self, _device_id: &PCWSTR, _key: &PROPERTYKEY) -> Result<()> {
        Ok(())
    }
}