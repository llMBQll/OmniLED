use log::{debug, warn};
use mlua::{ErrorContext, FromLua, Function, Lua, UserData, Value};
use num_traits::Unsigned;
use omni_led_derive::FromLuaValue;
use rusb::{DeviceHandle, GlobalContext};
use std::time::Duration;

use crate::devices::device::{Device, MemoryLayout, Settings, Size};
use crate::renderer::buffer::Buffer;

pub struct USBDevice {
    name: String,
    size: Size,
    settings: USBSettings,
    transform: Option<Function>,
    handle: DeviceHandle<GlobalContext>,
    layout: MemoryLayout,
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

impl Device for USBDevice {
    fn init(lua: &Lua, settings: Value) -> mlua::Result<Self> {
        let settings = USBDeviceSettings::from_lua(settings, lua)?;

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
                )));
            }
        };

        let handle = match device.open() {
            Ok(handle) => handle,
            Err(err) => return Err(mlua::Error::runtime(format!("{err}"))),
        };

        let interface = settings.usb_settings.interface;
        let alternate_setting = settings.usb_settings.alternate_setting;

        match handle.kernel_driver_active(interface) {
            Ok(true) => handle.detach_kernel_driver(interface).unwrap(),
            _ => {}
        };

        handle.claim_interface(interface).unwrap();
        handle
            .set_alternate_setting(interface, alternate_setting)
            .unwrap();

        Ok(Self {
            name: settings.name,
            size: settings.screen_size,
            settings: settings.usb_settings.clone(),
            transform: settings.transform,
            handle,
            layout: settings.memory_layout.unwrap(),
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

    fn memory_layout(&mut self, _lua: &Lua) -> mlua::Result<MemoryLayout> {
        Ok(self.layout)
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

#[derive(FromLuaValue, Clone)]
pub struct USBDeviceSettings {
    pub name: String,
    pub screen_size: Size,
    pub usb_settings: USBSettings,
    pub transform: Option<Function>,
    #[mlua(deprecated = memory_layout)]
    pub memory_representation: Option<MemoryLayout>,
    pub memory_layout: Option<MemoryLayout>,
}

impl Settings for USBDeviceSettings {
    type DeviceType = USBDevice;

    fn name(&self) -> String {
        self.name.clone()
    }
}

impl UserData for USBDeviceSettings {}

#[derive(FromLuaValue, Clone)]
pub struct USBSettings {
    #[mlua(transform = from_hex)]
    pub vendor_id: u16,
    #[mlua(transform = from_hex)]
    pub product_id: u16,
    #[mlua(transform = from_hex)]
    pub interface: u8,
    #[mlua(transform = from_hex)]
    pub alternate_setting: u8,
    #[mlua(transform = from_hex)]
    pub request_type: u8,
    #[mlua(transform = from_hex)]
    pub request: u8,
    #[mlua(transform = from_hex)]
    pub value: u16,
    #[mlua(transform = from_hex)]
    pub index: u16,
}

impl UserData for USBSettings {}

fn from_hex<T: Unsigned>(hex_value: String, _lua: &Lua) -> mlua::Result<T> {
    const HEX_PREFIX: &str = "0x";

    if !hex_value.starts_with(HEX_PREFIX) {
        return Err(mlua::Error::runtime(format!(
            "Hex number shall have a {HEX_PREFIX} prefix"
        )));
    }

    T::from_str_radix(&hex_value[2..], 16).map_err(move |_err| {
        mlua::Error::runtime(format!("Could not parse {} as hex value", hex_value))
    })
}
