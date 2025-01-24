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

    let (tx, mut rx): (Sender<AudioData>, Receiver<AudioData>) = mpsc::channel(256);

    let handle = Handle::current();
    let _audio = Audio::new(tx, handle);

    while let Some(data) = rx.recv().await {
        if let Some(name) = &data.name {
            debug!(
                "New default device: {}, volume: {}%, muted: {}",
                name, data.volume, data.is_muted
            );
        }

        plugin.update(data.into()).await.unwrap();
    }

    Ok(())
}

#[derive(IntoProto)]
#[proto(rename_all = PascalCase)]
struct AudioData {
    is_muted: bool,
    volume: i32,
    name: Option<String>,
}

impl AudioData {
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
