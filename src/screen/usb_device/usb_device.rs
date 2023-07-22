use std::time::Duration;
use log::{debug, warn};
use rusb::{DeviceHandle, GlobalContext};
use crate::screen::screen::{Screen, Size};
use crate::screen::screen::Error::InitFailed;
use crate::screen::screen::Result;
use crate::screen::supported_devices::device_info::{USBDeviceSettings, USBSettings};

pub struct USBDevice {
    name: String,
    size: Size,
    settings: USBSettings,
    handle: DeviceHandle<GlobalContext>,
}

impl USBDevice {
    pub fn new(settings: USBDeviceSettings) -> Result<Self> {
        let vendor_id = settings.usb_settings.vendor_id;
        let product_id = settings.usb_settings.product_id;

        let device = rusb::devices().unwrap().iter().find(|device| {
            let desc = device.device_descriptor().unwrap();
            desc.vendor_id() == vendor_id && desc.product_id() == product_id
        });

        let device = match device {
            Some(device) => device,
            None => return Err(InitFailed(format!("Failed to match vendor_id {:#06x} and product_id {:#06x}", vendor_id, product_id)))
        };

        let mut handle = match device.open() {
            Ok(handle) => handle,
            Err(err) => return Err(InitFailed(format!("{err}")))
        };

        let interface = settings.usb_settings.interface;

        match handle.kernel_driver_active(interface) {
            Ok(true) => handle.detach_kernel_driver(interface).unwrap(),
            _ => {}
        };

        handle.claim_interface(interface).unwrap();
        handle.set_alternate_setting(interface, 0).unwrap();

        Ok(Self {
            name: settings.name,
            size: settings.screen_size,
            settings: settings.usb_settings.clone(),
            handle,
        })
    }
}

impl Screen for USBDevice {
    fn init(&mut self) -> Result<()> {
        Ok(())
    }

    fn size(&mut self) -> Result<Size> {
        Ok(self.size)
    }

    fn update(&mut self, pixels: &Vec<u8>) -> Result<()> {
        let mut pixels = pixels.clone();
        let mut buf = Vec::with_capacity(pixels.len() + 2);
        buf.push(0x61);
        buf.append(&mut pixels);
        buf.push(0x00);

        self.handle.write_control(
            0x21,
            0x09,
            0x0300,
            0x01,
            buf.as_slice(),
            Duration::from_secs(1),
        ).unwrap();

        Ok(())
    }

    fn name(&self) -> Result<String> {
        Ok(self.name.clone())
    }
}

impl Drop for USBDevice {
    fn drop(&mut self) {
        let interface = self.settings.interface;

        match self.handle.release_interface(interface) {
            Ok(_) => {
                debug!("Released interface for {}", self.name);
                match self.handle.kernel_driver_active(self.settings.interface) {
                    Ok(false) => match self.handle.attach_kernel_driver(self.settings.interface) {
                        Ok(_) => debug!("Reattached kernel driver for {}", self.name),
                        Err(err) => warn!("Failed to reattach kernel driver for {}: {}", self.name, err)
                    }
                    _ => debug!("Kernel driver was already attached for {}", self.name)
                }
            }
            Err(err) => warn!("Failed to release interface for {}: {}", self.name, err)
        }
    }
}