use crate::renderer::buffer::Buffer;
use log::{debug, warn};
use mlua::{Lua, OwnedFunction, Value};
use rusb::{DeviceHandle, GlobalContext};
use std::time::Duration;

use crate::screen::screen::{MemoryRepresentation, Screen, Settings, Size};
use crate::screen::usb_device::usb_device_settings::{USBDeviceSettings, USBSettings};

pub struct USBDevice {
    name: String,
    size: Size,
    settings: USBSettings,
    transform: Option<OwnedFunction>,
    handle: DeviceHandle<GlobalContext>,
    representation: MemoryRepresentation,
}

impl USBDevice {
    fn write_bytes(&self, bytes: &[u8]) {
        self.handle
            .write_control(
                self.settings.request_type,
                self.settings.request,
                self.settings.value,
                self.settings.index,
                bytes,
                Duration::from_millis(10),
            )
            .unwrap();
    }
}

impl Screen for USBDevice {
    fn init(lua: &Lua, settings: Value) -> mlua::Result<Self> {
        let settings = USBDeviceSettings::new(lua, settings)?;

        let vendor_id = settings.usb_settings.vendor_id;
        let product_id = settings.usb_settings.product_id;

        let device = rusb::devices().unwrap().iter().find(|device| {
            let desc = device.device_descriptor().unwrap();
            desc.vendor_id() == vendor_id && desc.product_id() == product_id
        });

        let device = match device {
            Some(device) => device,
            None => {
                return Err(mlua::Error::runtime(format!(
                    "Failed to match vendor_id {:#06x} and product_id {:#06x}",
                    vendor_id, product_id
                )))
            }
        };

        #[allow(unused_mut)]
        let mut handle = match device.open() {
            Ok(handle) => handle,
            Err(err) => return Err(mlua::Error::runtime(format!("{err}"))),
        };

        let interface = settings.usb_settings.interface;
        let interface_setting = settings.usb_settings.interface_setting;

        match handle.kernel_driver_active(interface) {
            Ok(true) => handle.detach_kernel_driver(interface).unwrap(),
            _ => {}
        };

        handle.claim_interface(interface).unwrap();
        handle
            .set_alternate_setting(interface, interface_setting)
            .unwrap();

        Ok(Self {
            name: settings.name,
            size: settings.screen_size,
            settings: settings.usb_settings.clone(),
            transform: settings.transform,
            handle,
            representation: settings.memory_representation,
        })
    }

    fn size(&mut self, _: &Lua) -> mlua::Result<Size> {
        Ok(self.size)
    }

    fn update(&mut self, _: &Lua, buffer: Buffer) -> mlua::Result<()> {
        match &self.transform {
            Some(transform) => {
                let bytes: Vec<u8> = transform.call(buffer)?;
                self.write_bytes(bytes.as_slice())
            }
            None => self.write_bytes(buffer.bytes()),
        };

        Ok(())
    }

    fn name(&mut self, _: &Lua) -> mlua::Result<String> {
        Ok(self.name.clone())
    }

    fn memory_representation(&mut self, _lua: &Lua) -> mlua::Result<MemoryRepresentation> {
        Ok(self.representation)
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
                        Err(err) => warn!(
                            "Failed to reattach kernel driver for {}: {}",
                            self.name, err
                        ),
                    },
                    _ => debug!("Kernel driver was already attached for {}", self.name),
                }
            }
            Err(err) => warn!("Failed to release interface for {}: {}", self.name, err),
        }
    }
}
