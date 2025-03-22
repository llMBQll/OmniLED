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

use audio::Audio;
use clap::Parser;
use log::debug;
use omni_led_api::plugin::Plugin;
use omni_led_derive::IntoProto;
use std::error::Error;
use tokio::runtime::Handle;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

mod audio;

const NAME: &str = "AUDIO";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let options = Options::parse();
    let mut plugin = Plugin::new(NAME, &options.address).await?;

    let (tx, mut rx): (
        Sender<(DeviceData, DeviceType)>,
        Receiver<(DeviceData, DeviceType)>,
    ) = mpsc::channel(256);

    let handle = Handle::current();
    let _audio = Audio::new(tx, handle);

    while let Some((data, device_type)) = rx.recv().await {
        if let Some(name) = &data.name {
            debug!(
                "{:?} device: '{}', volume: {}%, muted: {}",
                device_type, name, data.volume, data.is_muted
            );
        }

        let event = match device_type {
            DeviceType::Input => AudioEvent {
                input_is_muted: Some(data.is_muted),
                input_volume: Some(data.volume),
                input_name: data.name,
                output_is_muted: None,
                output_volume: None,
                output_name: None,
            },
            DeviceType::Output => AudioEvent {
                input_is_muted: None,
                input_volume: None,
                input_name: None,
                output_is_muted: Some(data.is_muted),
                output_volume: Some(data.volume),
                output_name: data.name,
            },
        };

        plugin.update(event.into()).await.unwrap();
    }

    Ok(())
}

#[derive(IntoProto)]
#[proto(rename_all = PascalCase)]
struct AudioEvent {
    input_is_muted: Option<bool>,
    input_volume: Option<i32>,
    input_name: Option<String>,
    output_is_muted: Option<bool>,
    output_volume: Option<i32>,
    output_name: Option<String>,
}

#[derive(Copy, Clone, Debug)]
pub enum DeviceType {
    Input,
    Output,
}

struct DeviceData {
    is_muted: bool,
    volume: i32,
    name: Option<String>,
}

impl DeviceData {
    pub fn new(is_muted: bool, volume: i32, name: Option<String>) -> Self {
        Self {
            is_muted,
            volume,
            name,
        }
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Options {
    #[clap(short, long)]
    address: String,
}
