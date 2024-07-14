use clap::Parser;
use oled_api::{LogLevel, Plugin};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::media::session_data::SessionData;
use crate::media::Media;
use crate::Mode::{Both, Focused, Individual};

mod media;

const NAME: &str = "MEDIA";

#[tokio::main]
async fn main() {
    let options = Options::parse();
    let mut plugin = Plugin::new(NAME, &options.address).await.unwrap();

    let (tx, mut rx): (Sender<Message>, Receiver<Message>) = mpsc::channel(256);

    let mut map: HashMap<String, String> = HashMap::from_iter(options.map.into_iter());
    for (from, to) in &map {
        log_mapping(&tx, &from, &to).await;
    }
    let mode = options.mode;

    let media = Media::new(tx.clone());

    let loop_handle = tokio::task::spawn(async move {
        while let Some(message) = rx.recv().await {
            match message {
                Message::Event(current, name, session_data) => {
                    if current && (mode == Focused || mode == Both) {
                        plugin.update(session_data.clone().into()).await.unwrap();
                    }

                    if mode == Individual || mode == Both {
                        let transformed = match map.get(&name) {
                            Some(transformed) => transformed,
                            None => {
                                let transformed = transform_name(&tx, &name).await;
                                map.entry(name).or_insert(transformed)
                            }
                        };

                        plugin
                            .update_with_name(transformed, session_data.into())
                            .await
                            .unwrap();
                    }
                }
                Message::Log(message, level) => {
                    plugin.log(&message, level).await.unwrap();
                }
            }
        }
    });

    media.run().await;

    loop_handle.await.unwrap();
}

#[derive(Clone)]
enum Message {
    Event(bool, String, SessionData),
    Log(String, LogLevel),
}

async fn transform_name(tx: &Sender<Message>, name: &String) -> String {
    let mut new_name = String::with_capacity(name.capacity());

    for character in name.chars() {
        if character.is_ascii_alphanumeric() {
            new_name.push(character.to_ascii_uppercase())
        } else {
            new_name.push('_')
        }
    }

    if new_name.is_empty() || new_name.starts_with(|x: char| x.is_ascii_digit()) {
        new_name.insert(0, '_')
    }

    log_mapping(tx, &name, &new_name).await;

    new_name
}

async fn log_mapping(tx: &Sender<Message>, old: &str, new: &str) {
    tx.send(Message::Log(
        format!("Mapped '{}' to '{}'", old, new),
        LogLevel::Info,
    ))
    .await
    .unwrap();
}

#[derive(clap::Parser, Debug)]
#[command(author, version, about)]
struct Options {
    #[clap(short, long)]
    address: String,

    #[clap(long, value_parser = parse_pair)]
    map: Vec<(String, String)>,

    #[clap(short, long, value_parser = clap::value_parser!(Mode), default_value = "both", ignore_case = true)]
    mode: Mode,
}

#[derive(clap::ValueEnum, Clone, Debug, PartialEq)]
enum Mode {
    Individual,
    Focused,
    Both,
}

fn parse_pair(
    s: &str,
) -> Result<(String, String), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let pos = s
        .rfind('=')
        .ok_or_else(|| format!("invalid KEY=VALUE: no `=` found in `{s}`"))?;

    let key = &s[..pos];
    let value = &s[pos + 1..];

    if !Plugin::is_valid_identifier(value) {
        return Err("Key is not a valid event name".into());
    }

    Ok((key.to_string(), value.to_string()))
}
