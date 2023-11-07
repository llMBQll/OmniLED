use api::Api;
use clap::Parser;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use crate::media::session_data::SessionData;
use crate::media::Media;
use crate::Mode::{Both, Focused, Individual};

mod media;

const NAME: &str = "MEDIA";
static API: OnceLock<Api> = OnceLock::new();

#[tokio::main]
async fn main() {
    let options = Options::parse();

    API.set(Api::new(&options.address, NAME)).unwrap();

    let mut map: HashMap<String, String> = HashMap::from_iter(options.map.into_iter());
    let mode = options.mode;
    let media = Media::new(Arc::new(Mutex::new(
        move |name: &String, session_data: &SessionData, current: bool| {
            if current && (mode == Focused || mode == Both) {
                API.get().unwrap().update(session_data)
            }

            if mode == Individual || mode == Both {
                let name = map
                    .entry(name.clone())
                    .or_insert_with(|| transform_name(name));

                API.get().unwrap().update_with_name(session_data, name);
            }
        },
    )));

    media.run().await;
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

    println!("Mapped {} to {}", name, new_name);

    new_name
}

#[derive(clap::Parser, Debug)]
#[command(author, version, about)]
struct Options {
    #[clap(short, long)]
    address: String,

    #[clap(long, value_parser=parse_pair)]
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
        .find('=')
        .ok_or_else(|| format!("invalid KEY=VALUE: no `=` found in `{s}`"))?;

    //TODO validate value to be uppercase alphanumeric string

    Ok((s[..pos].to_owned(), s[pos + 1..].to_owned()))
}
