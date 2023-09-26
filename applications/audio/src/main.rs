use api::Api;
use audio::Audio;
use serde::Serialize;
use std::{env, sync::OnceLock, thread, time};

mod audio;

const NAME: &str = "AUDIO";

static API: OnceLock<Api> = OnceLock::new();

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let address = args[1].as_str();

    API.set(Api::new(String::from(address), String::from(NAME)))
        .unwrap();

    let _audio = Audio::new(|muted, volume, name| {
        API.get()
            .unwrap()
            .update(&AudioData::new(muted, volume, name));
    });

    thread::sleep(time::Duration::MAX);
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct AudioData {
    is_muted: bool,
    volume: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
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
