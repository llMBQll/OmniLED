use std::sync::mpsc;

use audio::Audio;
use log::debug;
use omni_led_api::plugin::Plugin;
use omni_led_derive::plugin_entry;
use serde::Serialize;

mod audio;

#[plugin_entry]
pub fn omni_led_run(plugin: Plugin, _args: Vec<&str>) {
    let (tx, rx) = mpsc::channel::<(DeviceData, DeviceType)>();

    let _audio = Audio::new(tx);

    while let Ok((data, device_type)) = rx.recv() {
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

        match device_type {
            DeviceType::Input => plugin
                .update(&InputAudioEvent { input: event_data })
                .unwrap(),
            DeviceType::Output => plugin
                .update(&OutputAudioEvent { output: event_data })
                .unwrap(),
        };
    }
}

#[derive(Copy, Clone, Debug)]
pub enum DeviceType {
    Input,
    Output,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct InputAudioEvent {
    input: Option<EventData>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct OutputAudioEvent {
    output: Option<EventData>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct EventData {
    is_muted: bool,
    volume: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
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
