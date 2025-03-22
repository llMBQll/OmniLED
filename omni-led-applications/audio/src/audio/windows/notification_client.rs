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

use crate::DeviceType;
use windows::Win32::Foundation::PROPERTYKEY;
use windows::Win32::Media::Audio::{
    DEVICE_STATE, EDataFlow, ERole, IMMNotificationClient, IMMNotificationClient_Impl, eCapture,
    eMultimedia, eRender,
};
use windows::core::{PCWSTR, Result, implement};

#[implement(IMMNotificationClient)]
pub struct NotificationClient<T>
where
    T: Fn(DeviceType) + 'static,
{
    callback: T,
}

impl<T: Fn(DeviceType)> NotificationClient<T> {
    pub fn new(callback: T) -> IMMNotificationClient {
        let this = Self { callback };

        this.into()
    }
}

#[allow(non_snake_case)]
impl<T: Fn(DeviceType)> IMMNotificationClient_Impl for NotificationClient_Impl<T> {
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
        _device_id: &PCWSTR,
    ) -> Result<()> {
        if role == eMultimedia && flow == eCapture {
            (self.callback)(DeviceType::Input);
        } else if role == eMultimedia && flow == eRender {
            (self.callback)(DeviceType::Output);
        }

        Ok(())
    }

    fn OnPropertyValueChanged(&self, _device_id: &PCWSTR, _key: &PROPERTYKEY) -> Result<()> {
        Ok(())
    }
}
