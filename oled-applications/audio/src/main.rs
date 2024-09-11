use audio::Audio;
use oled_api::Plugin;
use oled_derive::IntoProto;
use std::env;
use std::error::Error;
use tokio::runtime::Handle;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

mod audio;

const NAME: &str = "AUDIO";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let address = args[1].as_str();
    let mut plugin = Plugin::new(NAME, address).await?;

    let path = plugin.get_data_dir().await.unwrap();
    oled_log::init(path.join("logging.log"));

    let (tx, mut rx): (Sender<AudioData>, Receiver<AudioData>) = mpsc::channel(256);

    let handle = Handle::current();
    let _audio = Audio::new(tx, handle);

    while let Some(data) = rx.recv().await {
        plugin.update(data.into()).await.unwrap();
    }

    Ok(())
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
