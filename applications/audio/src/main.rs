use api::types::Table;
use api::Api;
use audio::Audio;
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

impl Into<Table> for AudioData {
    fn into(self) -> Table {
        let mut table = Table::default();

        table
            .items
            .insert("IsMuted".to_string(), self.is_muted.into());
        table.items.insert("Volume".to_string(), self.volume.into());
        if let Some(name) = self.name {
            table.items.insert("Name".to_string(), name.into());
        }

        table
    }
}
