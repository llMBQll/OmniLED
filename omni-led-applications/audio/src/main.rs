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
                input: Some(data),
                output: None,
            },
            DeviceType::Output => AudioEvent {
                input: None,
                output: Some(data),
            },
        };

        plugin.update(event.into()).await.unwrap();
    }

    Ok(())
}

#[derive(Copy, Clone, Debug)]
pub enum DeviceType {
    Input,
    Output,
}

#[derive(IntoProto)]
#[proto(rename_all = PascalCase)]
struct AudioEvent {
    input: Option<DeviceData>,
    output: Option<DeviceData>,
}

#[derive(IntoProto)]
#[proto(rename_all = PascalCase)]
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
