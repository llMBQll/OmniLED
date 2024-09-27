use windows::core::{implement, Result, PCWSTR};
use windows::Win32::Media::Audio::{
    eMultimedia, eRender, EDataFlow, ERole, IMMNotificationClient, IMMNotificationClient_Impl,
    DEVICE_STATE,
};
use windows::Win32::UI::Shell::PropertiesSystem::PROPERTYKEY;

#[implement(IMMNotificationClient)]
pub struct NotificationClient<T>
where
    T: Fn(&PCWSTR) + 'static,
{
    callback: T,
}

impl<T: Fn(&PCWSTR)> NotificationClient<T> {
    pub fn new(callback: T) -> IMMNotificationClient {
        let this = Self { callback };

        this.into()
    }
}

#[allow(non_snake_case)]
impl<T: Fn(&PCWSTR)> IMMNotificationClient_Impl for NotificationClient_Impl<T> {
    fn OnDeviceStateChanged(&self, _device_id: &PCWSTR, _new_state: DEVICE_STATE) -> Result<()> {
        Ok(())
    }

    fn OnDeviceAdded(&self, _device_id: &PCWSTR) -> Result<()> {
        Ok(())
    }

    fn OnDeviceRemoved(&self, _device_id: &PCWSTR) -> Result<()> {
        Ok(())
    }

    fn OnDefaultDeviceChanged(
        &self,
        flow: EDataFlow,
        role: ERole,
        default_device_id: &PCWSTR,
    ) -> Result<()> {
        if flow != eRender || role != eMultimedia {
            return Ok(());
        }
        (self.callback)(default_device_id);
        Ok(())
    }

    fn OnPropertyValueChanged(&self, _device_id: &PCWSTR, _key: &PROPERTYKEY) -> Result<()> {
        Ok(())
    }
}
