use mlua::{ErrorContext, FromLua, Function, Lua, UserData, Value};
use num_traits::Unsigned;
use oled_derive::FromLuaValue;

use crate::devices::device::{MemoryRepresentation, Settings, Size};

#[derive(FromLuaValue, Clone)]
pub struct USBDeviceSettings {
    pub name: String,
    pub screen_size: Size,
    pub usb_settings: USBSettings,
    pub transform: Option<Function>,
    pub memory_representation: MemoryRepresentation,
}

impl Settings for USBDeviceSettings {
    fn new(lua: &Lua, value: Value) -> mlua::Result<Self> {
        Self::from_lua(value, lua)
    }
}

impl UserData for USBDeviceSettings {}

#[derive(FromLuaValue, Clone)]
pub struct USBSettings {
    #[mlua(transform(from_hex))]
    pub vendor_id: u16,
    #[mlua(transform(from_hex))]
    pub product_id: u16,
    #[mlua(transform(from_hex))]
    pub interface: u8,
    #[mlua(transform(from_hex))]
    pub endpoint: u8,
    #[mlua(transform(from_hex))]
    pub request_type: u8,
    #[mlua(transform(from_hex))]
    pub request: u8,
    #[mlua(transform(from_hex))]
    pub value: u16,
    #[mlua(transform(from_hex))]
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
