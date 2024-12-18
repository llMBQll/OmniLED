/*
 * OmniLED is a software for displaying data on various OLED devices.
 * Copyright (C) 2024  Michał Bałabanow <m.balabanow@gmail.com>
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

include!(concat!(env!("OUT_DIR"), "/binaries.rs"));

pub const APPLICATIONS: &[u8] = include_bytes!("../../config/applications.lua");
pub const DEVICES: &[u8] = include_bytes!("../../config/devices.lua");
pub const SCRIPTS: &[u8] = include_bytes!("../../config/scripts.lua");
pub const SETTINGS: &[u8] = include_bytes!("../../config/settings.lua");

pub const LICENSE: &[u8] = include_bytes!("../../LICENSE");
