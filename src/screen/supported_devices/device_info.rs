use serde::{Deserialize, Deserializer};

use crate::screen::screen::Size;

#[derive(Deserialize, Debug)]
pub struct DeviceInfo {
    pub name: String,
    pub screen_size: Size,
    pub usb_info: Option<USBInfo>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct USBInfo {
    #[serde(with = "hex")]
    pub vendor_id: Vec<u8>,
    #[serde(with = "hex")]
    pub product_id: Vec<u8>,
    #[serde(with = "hex")]
    pub interface: Vec<u8>,
    #[serde(with = "hex")]
    pub endpoint: Vec<u8>,
}