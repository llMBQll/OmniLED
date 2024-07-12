use audio::Audio;
use oled_api::Api;
use oled_derive::IntoProto;
use std::{env, sync::OnceLock, thread, time};

mod audio;

const NAME: &str = "AUDIO";

static API: OnceLock<Api> = OnceLock::new();

fn main() {
    let args: Vec<String> = env::args().collect();
    let address = args[1].as_str();

    API.set(Api::new(address, NAME)).unwrap();

    let _audio = Audio::new(|muted, volume, name| {
        API.get()
            .unwrap()
            .update(AudioData::new(muted, volume, name).into());
    });

    thread::sleep(time::Duration::MAX);
}

#[derive(IntoProto)]
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
