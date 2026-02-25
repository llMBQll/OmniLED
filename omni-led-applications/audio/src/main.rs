use audio::Audio;
use clap::Parser;
use log::debug;
use omni_led_api::plugin::Plugin;
use omni_led_api::types::Table;
use omni_led_derive::IntoProto;
use tokio::runtime::Handle;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

mod audio;

const NAME: &str = "AUDIO";

#[tokio::main]
async fn main() {
    let options = Options::parse();
    let mut plugin = Plugin::new(NAME, &options.address).await.unwrap();

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

        let event_data = if data.connected {
            Some(EventData {
                is_muted: data.is_muted,
                volume: data.volume,
                name: data.name,
            })
        } else {
            None
        };

        let event: Table = match device_type {
            DeviceType::Input => InputAudioEvent { input: event_data }.into(),
            DeviceType::Output => OutputAudioEvent { output: event_data }.into(),
        };

        plugin.update(event.into()).await.unwrap();
    }
}

#[derive(Copy, Clone, Debug)]
pub enum DeviceType {
    Input,
    Output,
}

#[derive(IntoProto)]
#[proto(rename_all = PascalCase)]
struct InputAudioEvent {
    #[proto(strong_none)]
    input: Option<EventData>,
}

#[derive(IntoProto)]
#[proto(rename_all = PascalCase)]
struct OutputAudioEvent {
    #[proto(strong_none)]
    output: Option<EventData>,
}

#[derive(IntoProto)]
#[proto(rename_all = PascalCase)]
struct EventData {
    is_muted: bool,
    volume: i32,
    name: Option<String>,
}

struct DeviceData {
    connected: bool,
    is_muted: bool,
    volume: i32,
    name: Option<String>,
}

impl DeviceData {
    pub fn new(connected: bool, is_muted: bool, volume: i32, name: Option<String>) -> Self {
        Self {
            connected,
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
