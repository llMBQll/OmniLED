use clap::Parser;
use log::info;
use omni_led_api::new_plugin;
use omni_led_api::plugin::Plugin;
use omni_led_api::rust_api::OmniLedApi;
use omni_led_api::types::Table;
use omni_led_derive::plugin_entry;
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::Mode::{Both, Focused, Individual};
use crate::media::Media;
use crate::media::session_data::SessionData;

mod media;

#[plugin_entry]
pub async fn omni_led_run(api: OmniLedApi, args: Vec<&str>) {
    let plugin = new_plugin!(api);
    let options = Options::parse_from(args);

    let (tx, mut rx): (Sender<Data>, Receiver<Data>) = mpsc::channel(256);

    let mut map: HashMap<String, String> = HashMap::from_iter(options.map.into_iter());
    for (from, to) in &map {
        log_mapping(&from, &to);
    }
    let mode = options.mode;

    let loop_handle = tokio::task::spawn(async move {
        while let Some((current, name, session_data)) = rx.recv().await {
            let transformed = match map.get(&name) {
                Some(transformed) => transformed,
                None => {
                    let transformed = transform_name(&name);
                    map.entry(name).or_insert(transformed)
                }
            };
            let session_data = session_data_with_source(session_data, transformed);

            if current && (mode == Focused || mode == Both) {
                plugin.update(session_data.clone().into()).unwrap();
            }

            if mode == Individual || mode == Both {
                plugin
                    .update_with_name(transformed, session_data.into())
                    .unwrap();
            }
        }
    });

    Media::run(tx).await;

    loop_handle.await.unwrap();
}

type Data = (bool, String, SessionData);

fn session_data_with_source(session_data: SessionData, source: &str) -> Table {
    let mut table: Table = session_data.into();
    table.items.insert("Source".into(), source.into());
    table
}

fn transform_name(name: &String) -> String {
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

    log_mapping(&name, &new_name);

    new_name
}

fn log_mapping(old: &str, new: &str) {
    info!("Mapped '{}' to '{}'", old, new);
}

#[derive(clap::Parser, Debug)]
#[command(author, version, about)]
struct Options {
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
