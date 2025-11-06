/*
 * OmniLED is a software for displaying data on various OLED devices.
 * Copyright (C) 2025  Michał Bałabanow <m.balabanow@gmail.com>
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

use log::info;
use mlua::{FromLua, Lua};
use omni_led_lib::common::common::load_internal_functions;
use omni_led_lib::common::user_data::UserDataRef;
use omni_led_lib::constants::configs::{ConfigType, Configs};
use omni_led_lib::constants::constants::Constants;
use omni_led_lib::devices::devices::Devices;
use omni_led_lib::devices::usb_device::usb_device::USBDeviceSettings;
use omni_led_lib::logging::logger::Log;

pub const STEEL_SERIES_VENDOR_ID: u16 = 0x1038;

pub fn load_supported_devices(lua: &Lua) {

    // let available: Vec<_> = UserDataRef::<Devices>::load(&lua)
    //     .get()
    //     .get_available_settings()
    //     .into_iter()
    //     .filter_map(|value| match USBDeviceSettings::from_lua(value, &lua) {
    //         Ok(settings) => Some((
    //             settings.name,
    //             settings.usb_settings.vendor_id,
    //             settings.usb_settings.product_id,
    //         )),
    //         Err(_) => None,
    //     })
    //     .collect();
    //
    // info!("Available USB devices:");
    // for (name, vendor_id, product_id) in available {
    //     info!("  {name} {:04X}:{:04X}", vendor_id, product_id)
    // }
}
