use std::sync::mpsc::Sender;
use windows::Win32::Media::Audio::AUDIO_VOLUME_NOTIFICATION_DATA;
use windows::Win32::Media::Audio::Endpoints::{
    IAudioEndpointVolumeCallback, IAudioEndpointVolumeCallback_Impl,
};
use windows::core::implement;

use crate::{DeviceData, DeviceType};

#[implement(IAudioEndpointVolumeCallback)]
pub struct AudioEndpointVolumeCallback {
    tx: Sender<(DeviceData, DeviceType)>,
    device_type: DeviceType,
}

impl AudioEndpointVolumeCallback {
    pub(crate) fn new(
        tx: Sender<(DeviceData, DeviceType)>,
        device_type: DeviceType,
    ) -> IAudioEndpointVolumeCallback {
        Self { tx, device_type }.into()
    }
}

#[allow(non_snake_case)]
impl IAudioEndpointVolumeCallback_Impl for AudioEndpointVolumeCallback_Impl {
    fn OnNotify(&self, pnotify: *mut AUDIO_VOLUME_NOTIFICATION_DATA) -> windows::core::Result<()> {
        if pnotify.is_null() {
            return Ok(());
        }

        let data = &unsafe { *pnotify };

        let muted = data.bMuted.into();
        let volume = (data.fMasterVolume * 100.0).round() as i32;

        self.tx
            .send((DeviceData::new(true, muted, volume, None), self.device_type))
            .unwrap();

        Ok(())
    }
}
