use tokio::runtime::Handle;
use tokio::sync::mpsc::Sender;
use windows::Win32::Media::Audio::AUDIO_VOLUME_NOTIFICATION_DATA;
use windows::Win32::Media::Audio::Endpoints::{
    IAudioEndpointVolumeCallback, IAudioEndpointVolumeCallback_Impl,
};
use windows::core::implement;

use crate::{DeviceData, DeviceType};

#[implement(IAudioEndpointVolumeCallback)]
pub struct AudioEndpointVolumeCallback {
    tx: Sender<(DeviceData, DeviceType)>,
    handle: Handle,
    device_type: DeviceType,
}

impl AudioEndpointVolumeCallback {
    pub(crate) fn new(
        tx: Sender<(DeviceData, DeviceType)>,
        handle: Handle,
        device_type: DeviceType,
    ) -> IAudioEndpointVolumeCallback {
        let this = Self {
            tx,
            handle,
            device_type,
        };

        this.into()
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

        let tx = self.tx.clone();
        let device_type = self.device_type;
        self.handle.spawn(async move {
            tx.send((DeviceData::new(muted, volume, None), device_type))
                .await
                .unwrap();
        });

        Ok(())
    }
}
