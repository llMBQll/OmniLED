use audio::Audio;
use clap::Parser;
use oled_api::Plugin;
use oled_derive::IntoProto;
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
#[proto(rename_all(PascalCase))]
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
