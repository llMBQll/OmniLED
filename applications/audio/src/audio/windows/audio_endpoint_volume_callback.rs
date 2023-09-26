use windows::core::implement;
use windows::Win32::Media::Audio::Endpoints::{
    IAudioEndpointVolumeCallback, IAudioEndpointVolumeCallback_Impl,
};
use windows::Win32::Media::Audio::AUDIO_VOLUME_NOTIFICATION_DATA;

#[implement(IAudioEndpointVolumeCallback)]
pub struct AudioEndpointVolumeCallback {
    callback: fn(bool, i32, Option<String>),
}

impl AudioEndpointVolumeCallback {
    pub fn new(callback: fn(bool, i32, Option<String>)) -> IAudioEndpointVolumeCallback {
        let this = Self { callback };

        this.into()
    }
}

#[allow(non_snake_case)]
impl IAudioEndpointVolumeCallback_Impl for AudioEndpointVolumeCallback {
    fn OnNotify(&self, pnotify: *mut AUDIO_VOLUME_NOTIFICATION_DATA) -> windows::core::Result<()> {
        let data = unsafe { &*pnotify as &AUDIO_VOLUME_NOTIFICATION_DATA };

        let muted = data.bMuted.into();
        let volume = (data.fMasterVolume * 100.0).round() as i32;

        (self.callback)(muted, volume, None);

        Ok(())
    }
}
