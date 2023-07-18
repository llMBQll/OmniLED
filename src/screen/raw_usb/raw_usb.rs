use std::time::Duration;
use rusb::{DeviceHandle, GlobalContext};
use crate::screen::screen::{Screen, Size};
use crate::screen::screen::Error::{DeviceNotSupported, InitFailed};
use crate::screen::screen::Result;
use crate::screen::supported_devices::device_info::USBInfo;
use crate::screen::supported_devices::supported_devices::get_supported_devices;

pub struct RawUSB {
    name: String,
    size: Size,
    info: USBInfo,
    handle: DeviceHandle<GlobalContext>,
}

impl RawUSB {
    pub fn new(name: String) -> Result<Self> {
        let device_info = match get_supported_devices().iter().find(|&x| x.name == *name) {
            Some(device_info) => device_info,
            None => return Err(DeviceNotSupported),
        };

        let usb_info = match &device_info.usb_info {
            Some(usb_info) => usb_info,
            None => return Err(DeviceNotSupported)
        };
        let vendor_id = usb_info.vendor_id;
        let product_id = usb_info.product_id;

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

        match handle.kernel_driver_active(usb_info.interface) {
            Ok(true) => handle.detach_kernel_driver(usb_info.interface).unwrap(),
            _ => {}
        };

        handle.claim_interface(usb_info.interface).unwrap();
        handle.set_alternate_setting(usb_info.interface, 0).unwrap();

        Ok(Self {
            name,
            size: device_info.screen_size,
            info: usb_info.clone(),
            handle,
        })
    }
}

impl Screen for RawUSB {
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