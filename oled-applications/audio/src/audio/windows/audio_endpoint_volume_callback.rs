use tokio::runtime::Handle;
use tokio::sync::mpsc::Sender;
use windows::core::implement;
use windows::Win32::Media::Audio::Endpoints::{
    IAudioEndpointVolumeCallback, IAudioEndpointVolumeCallback_Impl,
};
use windows::Win32::Media::Audio::AUDIO_VOLUME_NOTIFICATION_DATA;

use crate::AudioData;

#[implement(IAudioEndpointVolumeCallback)]
pub struct AudioEndpointVolumeCallback {
    tx: Sender<AudioData>,
    handle: Handle,
}

impl AudioEndpointVolumeCallback {
    pub(crate) fn new(tx: Sender<AudioData>, handle: Handle) -> IAudioEndpointVolumeCallback {
        let this = Self { tx, handle };

        this.into()
    }
}

#[allow(non_snake_case)]
impl IAudioEndpointVolumeCallback_Impl for AudioEndpointVolumeCallback_Impl {
    fn OnNotify(&self, pnotify: *mut AUDIO_VOLUME_NOTIFICATION_DATA) -> windows::core::Result<()> {
        let data = unsafe { &*pnotify };

        let muted = data.bMuted.into();
        let volume = (data.fMasterVolume * 100.0).round() as i32;

        let tx = self.tx.clone();
        self.handle.spawn(async move {
            tx.send(AudioData::new(muted, volume, None)).await.unwrap();
        });

        Ok(())
    }
}
