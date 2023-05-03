use std::collections::hash_map::{DefaultHasher, Entry};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};
use mlua::Lua;

use serde::{Deserialize, Serialize};
use tokio::time::{Duration, Instant};
use warp::Filter;
use warp::sse::Event;
use crate::application_loader::application_loader::ApplicationLoader;

use crate::ApplicationMetadataReplyData::{Reason, Token};
use crate::events::events::Events;
use crate::keyboard_api::KeyboardAPI;
use crate::logging::logger::Logger;
// use crate::model::display::Display;
// use crate::model::operation::Operation;
// use crate::plugin::plugin::Plugin;
use crate::renderer::renderer::Renderer;
use crate::settings::settings::Settings;

mod application_loader;
mod events;
mod keyboard_api;
mod logging;
mod model;
mod plugins;
mod renderer;
mod script_handler;
mod server;
mod settings;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ApplicationMetadata {
    name: String,
    timeout_ms: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
enum ApplicationMetadataReplyData {
    Token(u64),
    Reason(String),
}

#[derive(Serialize)]
struct ApplicationMetadataReply {
    #[serde(flatten)]
    metadata: ApplicationMetadata,
    #[serde(flatten)]
    data: ApplicationMetadataReplyData,
}

#[derive(Deserialize, Serialize)]
struct Update {
    #[serde(flatten)]
    fields: HashMap<String, serde_json::Value>,
}

#[derive(Deserialize, Serialize)]
struct UpdateReply {
    timeout_in: u64,
}

#[derive(Deserialize, Serialize)]
struct GenericErr {
    reason: String,
}

struct Applications {
    applications: HashMap<u64, (String, Instant)>,
}

impl Applications {
    fn new() -> Self {
        Applications {
            applications: HashMap::new()
        }
    }

    fn register(&mut self, name: &String) -> Option<u64> {
        let token = Self::hash(name);
        let now = Instant::now();
        let mut valid = false;

        self.applications.entry(token).and_modify(|(current, timeout)| {
            if now > *timeout {
                *current = name.clone();
                *timeout = now + Duration::from_secs(30);
                valid = true;
            }
        }).or_insert_with(|| {
            valid = true;
            (name.clone(), now + Duration::from_secs(30))
        });

        match valid {
            true => Some(token),
            false => None
        }
    }

    fn update(&mut self, token: u64) -> Option<(String, Instant)> {
        let now = Instant::now();
        let mut valid = false;

        let entry = self.applications.entry(token).and_modify(|(_, timeout)| {
            if now < *timeout {
                *timeout = now + Duration::from_secs(30);
                valid = true;
            }
        });

        match entry {
            Entry::Occupied(x) => {
                if valid {
                    let (name, timeout) = x.get();
                    Some((name.clone(), timeout.clone()))
                } else {
                    None
                }
            }
            Entry::Vacant(_) => {
                None
            }
        }
    }

    fn hash<T: Hash>(t: &T) -> u64 {
        // TODO randomize hash on every startup

        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }
}

fn main() {
    let lua = Lua::new();

    let _logger = Logger::new(&lua);
    Events::load(&lua);
    Settings::load(&lua);

    // let application_loader = ApplicationLoader::new(&lua);
    // application_loader.load_applications().unwrap();
}

// #[tokio::main]
// async fn main() {
//     let renderer = setup_renderer();
//     let keyboard_api = setup_keyboard_api();
//
//     run_server(renderer, keyboard_api).await;
// }

fn _setup_keyboard_api() -> KeyboardAPI {
    KeyboardAPI::new()
}

fn _setup_renderer() -> Renderer {
    // TODO: allow to change screen size dynamically
    // TODO: expose screen dimensions as lisp environment variables

    const WIDTH: usize = 128;
    const HEIGHT: usize = 40;

    Renderer::new(HEIGHT, WIDTH)
}

async fn _run_server(_renderer: Renderer, _keyboard_api: KeyboardAPI) {
    let applications_src = Arc::new(Mutex::new(Applications::new()));
    let queue_src = Arc::new(Mutex::new(Vec::new()));

    let applications = Arc::clone(&applications_src);
    let register = warp::post().and(warp::path("register")).and(warp::body::json()).map(move |metadata: ApplicationMetadata| {
        match applications.lock().unwrap().register(&metadata.name) {
            Some(token) => {
                let reply = warp::reply::json(&ApplicationMetadataReply {
                    metadata,
                    data: Token(token),
                });
                warp::reply::with_status(reply, warp::http::StatusCode::OK)
            }
            None => {
                let reply = warp::reply::json(&ApplicationMetadataReply {
                    metadata,
                    data: Reason(String::from("Application with the same name is already registered.")),
                });
                warp::reply::with_status(reply, warp::http::StatusCode::BAD_REQUEST)
            }
        }
    });

    let applications = Arc::clone(&applications_src);
    let queue = Arc::clone(&queue_src);
    let update = warp::path!("update" / u64).and(warp::body::json()).map(move |token: u64, update: HashMap<String, serde_json::Value>| {
        match applications.lock().unwrap().update(token) {
            Some((name, timeout)) => {
                queue.lock().unwrap().push((name, update));

                let reply = warp::reply::json(&UpdateReply {
                    timeout_in: (timeout - Instant::now()).as_millis() as u64
                });
                warp::reply::with_status(reply, warp::http::StatusCode::OK)
            }
            None => {
                let reply = warp::reply::json(&GenericErr {
                    reason: String::from("Application not registered or timed out.")
                });
                warp::reply::with_status(reply, warp::http::StatusCode::BAD_REQUEST)
            }
        }
    });

    let applications = Arc::clone(&applications_src);
    let heartbeat = warp::path!("heartbeat" / u64).map(move |token: u64| {
        match applications.lock().unwrap().update(token) {
            Some((_, timeout)) => {
                let reply = warp::reply::json(&UpdateReply {
                    timeout_in: (timeout - Instant::now()).as_millis() as u64
                });
                warp::reply::with_status(reply, warp::http::StatusCode::OK)
            }
            None => {
                let reply = warp::reply::json(&GenericErr {
                    reason: String::from("Application not registered or timed out.")
                });
                warp::reply::with_status(reply, warp::http::StatusCode::BAD_REQUEST)
            }
        }
    });

    let queue = Arc::clone(&queue_src);
    tokio::task::spawn(async move {
        const DURATION: Duration = Duration::from_millis(100);
        loop {
            let begin = Instant::now();

            // match env.update(&queue.lock().unwrap(), DURATION) {
            //     Ok(operations) => {
            //         if operations.len() > 0 {
            //             let image = renderer.render(operations);
            //             keyboard_api.set_image(&image);
            //         }
            //     }
            //     Err(error) => {
            //         println!("{}", error);
            //     }
            // }
            *queue.lock().unwrap() = Vec::new();

            let end = Instant::now();
            let update_duration = end - begin;
            tokio::time::sleep(DURATION.saturating_sub(update_duration)).await;
        }
    });

    let _paths = warp::post().and(register.or(update).or(heartbeat));

    // stop the program from exiting
    tokio::time::sleep(Duration::MAX).await;
}