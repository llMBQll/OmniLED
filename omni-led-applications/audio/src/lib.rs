use audio::Audio;
use log::debug;
use omni_led_api::new_plugin;
use omni_led_api::rust_api::OmniLedApi;
use omni_led_api::types::Table;
use omni_led_derive::{IntoProto, plugin_entry};
use tokio::runtime::Handle;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

mod audio;

#[plugin_entry]
pub async fn omni_led_run(api: OmniLedApi, _args: Vec<&str>) {
    let plugin = new_plugin!(api);

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

        plugin.update(event.into()).unwrap();
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
