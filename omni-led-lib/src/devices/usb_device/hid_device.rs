use mlua::{ErrorContext, FromLua, Function, Lua, UserData, Value};
use omni_led_derive::FromLuaValue;

use crate::devices::device::{Device, MemoryLayout, Settings, Size};
use crate::devices::usb_device::parse::from_hex;
use crate::renderer::buffer::Buffer;

pub struct HidDevice {
    name: String,
    size: Size,
    handle: hidapi::HidDevice,
    transform: Option<Function>,
    layout: MemoryLayout,
}

impl HidDevice {
    fn write_bytes(&self, bytes: &[u8]) {
        self.handle.send_feature_report(bytes).unwrap();
    }
}

impl Device for HidDevice {
    fn init(lua: &Lua, settings: Value) -> mlua::Result<Self> {
        let settings = HidDeviceSettings::from_lua(settings, lua)?;

        let vendor_id = settings.hid_settings.vendor_id;
        let product_id = settings.hid_settings.product_id;
        let interface = settings.hid_settings.interface;

        let api = hidapi::HidApi::new().unwrap();
        let device = api.device_list().find(|device| {
            device.vendor_id() == vendor_id
                && device.product_id() == product_id
                && device.interface_number() == interface as i32
        });

        let device = match device {
            Some(device) => device,
            None => {
                return Err(mlua::Error::runtime(format!(
                    "Failed to match vendor_id {:#06x} and product_id {:#06x} with interface {:#04x}",
                    vendor_id, product_id, interface,
                )));
            }
        };

        let handle = device.open_device(&api).map_err(mlua::Error::external)?;

        Ok(Self {
            name: settings.name,
            size: settings.screen_size,
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
                let mut bytes: Vec<u8> = transform.call(buffer)?;
                bytes.insert(0, 0x00);
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

#[derive(FromLuaValue, Clone)]
pub struct HidDeviceSettings {
    pub name: String,
    pub screen_size: Size,
    pub hid_settings: HidSettings,
    pub transform: Option<Function>,
    pub memory_layout: Option<MemoryLayout>,
}

impl Settings for HidDeviceSettings {
    type DeviceType = HidDevice;

    fn name(&self) -> String {
        self.name.clone()
    }
}

impl UserData for HidDeviceSettings {}

#[derive(FromLuaValue, Clone)]
pub struct HidSettings {
    #[mlua(transform = from_hex)]
    pub vendor_id: u16,
    #[mlua(transform = from_hex)]
    pub product_id: u16,
    #[mlua(transform = from_hex)]
    pub interface: u8,
}

impl UserData for HidSettings {}
