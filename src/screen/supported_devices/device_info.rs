use num_traits::Unsigned;
use serde::{Deserialize, Deserializer};
use serde::de::{Error, Visitor};
use std::fmt;
use std::marker::PhantomData;

use crate::screen::screen::Size;

#[derive(Deserialize, Debug)]
pub enum Output {
    SteelseriesEngineDevice(SteelseriesEngineDevice),
    USBDevice(USBDevice),
    // HTTP, Bluetooth etc ?
}

#[derive(Deserialize, Debug)]
pub struct SteelseriesEngineDevice {
    pub name: String,
    pub screen_size: Size,
}

#[derive(Deserialize, Debug)]
pub struct USBDevice {
    pub name: String,
    pub screen_size: Size,
    pub usb_settings: USBSettings,
}

#[derive(Deserialize, Debug, Clone)]
pub struct USBSettings {
    #[serde(deserialize_with = "from_hex")]
    pub vendor_id: u16,
    #[serde(deserialize_with = "from_hex")]
    pub product_id: u16,
    #[serde(deserialize_with = "from_hex")]
    pub interface: u8,
    #[serde(deserialize_with = "from_hex")]
    pub endpoint: u8,
}

struct HexVisitor<T: Unsigned> {
    _phantom: PhantomData<T>,
}

impl<'de, T: Unsigned> Visitor<'de> for HexVisitor<T> {
    type Value = T;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a hex string")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E> where E: Error {
        const HEX_PREFIX: &str = "0x";

        if !s.starts_with(HEX_PREFIX) {
            return Err(Error::custom(format!("Hex number shall have a {HEX_PREFIX} prefix")));
        }

        // TODO better error message
        Self::Value::from_str_radix(&s[2..], 16).map_err(|_| Error::custom("Could not parse the number"))
    }
}

fn from_hex<'de, D: Deserializer<'de>, T: Unsigned>(deserializer: D) -> Result<T, D::Error> {
    deserializer.deserialize_str(HexVisitor::<T> { _phantom: Default::default() })
}