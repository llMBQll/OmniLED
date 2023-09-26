use mlua::{FromLua, Function, IntoLua, Lua, LuaSerdeExt, OwnedFunction, Table, Value};
use num_traits::Unsigned;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer};
use std::fmt;
use std::marker::PhantomData;

use crate::screen::screen::{Error, Settings, Size};

pub struct USBDeviceSettings {
    pub name: String,
    pub screen_size: Size,
    pub usb_settings: USBSettings,
    pub transform: OwnedFunction,
}

impl Settings for USBDeviceSettings {
    fn new(lua: &Lua, value: Value) -> crate::screen::screen::Result<Self> {
        let table = match value {
            Value::Table(table) => table,
            _ => return Err(Error::WrongParameter(String::from("Table expected"))),
        };

        let name = get_value(&table, "name")?;
        let screen_size = get_value(&table, "screen_size")?;
        let screen_size: Size = lua
            .from_value(screen_size)
            .map_err(|err| Error::WrongParameter(format!("{err}")))?;
        let usb_settings = get_value(&table, "usb_settings")?;
        let usb_settings: USBSettings = lua
            .from_value(usb_settings)
            .map_err(|err| Error::WrongParameter(format!("{err}")))?;
        let transform: Function = get_value(&table, "transform")?;

        Ok(Self {
            name,
            screen_size,
            usb_settings,
            transform: transform.into_owned(),
        })
    }
}

fn get_value<'lua, K: IntoLua<'lua>, V: FromLua<'lua>>(
    table: &Table<'lua>,
    key: K,
) -> crate::screen::screen::Result<V> {
    table
        .get(key)
        .map_err(|err| Error::WrongParameter(format!("{err}")))
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
    #[serde(deserialize_with = "from_hex")]
    pub request_type: u8,
    #[serde(deserialize_with = "from_hex")]
    pub request: u8,
    #[serde(deserialize_with = "from_hex")]
    pub value: u16,
    #[serde(deserialize_with = "from_hex")]
    pub index: u16,
}

struct HexVisitor<T: Unsigned> {
    _phantom: PhantomData<T>,
}

impl<'de, T: Unsigned> Visitor<'de> for HexVisitor<T> {
    type Value = T;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a hex string")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        const HEX_PREFIX: &str = "0x";

        if !s.starts_with(HEX_PREFIX) {
            return Err(serde::de::Error::custom(format!(
                "Hex number shall have a {HEX_PREFIX} prefix"
            )));
        }

        // TODO better error message
        Self::Value::from_str_radix(&s[2..], 16)
            .map_err(|_| serde::de::Error::custom("Could not parse the number"))
    }
}

fn from_hex<'de, D: Deserializer<'de>, T: Unsigned>(deserializer: D) -> Result<T, D::Error> {
    deserializer.deserialize_str(HexVisitor::<T> {
        _phantom: Default::default(),
    })
}
