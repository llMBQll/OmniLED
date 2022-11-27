use std::collections::hash_map::{DefaultHasher, Entry};
use std::collections::HashMap;
use std::fs::File;
use std::hash::{Hash, Hasher};
use warp::Filter;
use serde::{Deserialize, Serialize};
use tokio::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use rust_lisp::model::RuntimeError;
use crate::ApplicationMetadataReplyData::{Reason, Token};

use crate::lisp_handler::lisp_handler::LispHandler;
use crate::keyboard_api::KeyboardAPI;
use crate::renderer::renderer::Renderer;
use crate::model::display::Display;
use crate::model::operation::Operation;

mod keyboard_api;
mod lisp_handler;
mod model;
mod renderer;

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

        self.applications.entry(token)
            .and_modify(|(current, timeout)| {
                if now > *timeout {
                    *current = name.clone();
                    *timeout = now + Duration::from_secs(30);
                    valid = true;
                }
            })
            .or_insert_with(|| {
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

        let entry = self.applications.entry(token)
            .and_modify(|(_, timeout)| {
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
                }
                else {
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

#[tokio::main]
async fn main() {
    // TODO load plugins

    let env = setup_env();
    let renderer = setup_renderer();
    let keyboard_api = setup_keyboard_api();
    // let _endpoints = setup_endpoints();

    run_server(env, renderer, keyboard_api).await;
}

fn setup_keyboard_api() -> KeyboardAPI {
    KeyboardAPI::new()
}

fn setup_env() -> LispHandler {
    let mut file = File::open("displays.json").unwrap();
    let displays: Vec<Display> = serde_json::from_reader(&mut file).unwrap();

    let mut env = LispHandler::new();
    env.register(displays).expect("Register displays failed");
    env
}

fn setup_renderer() -> Renderer {
    // TODO: allow to change screen size dynamically

    const WIDTH: usize = 128;
    const HEIGHT: usize = 40;

    Renderer::new(HEIGHT, WIDTH)
}

async fn run_server(mut env: LispHandler, mut renderer: Renderer, mut keyboard_api: KeyboardAPI) {
    let applications_src = Arc::new(Mutex::new(Applications::new()));
    let queue_src = Arc::new(Mutex::new(Vec::new()));
    // let env = Arc::new(Mutex::new(env));

    let applications = Arc::clone(&applications_src);
    let register = warp::post()
        .and(warp::path("register"))
        .and(warp::body::json())
        .map(move |metadata: ApplicationMetadata| {
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
    let update = warp::path!("update" / u64)
        .and(warp::body::json())
        .map(move |token: u64, update: HashMap<String, serde_json::Value>| {
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
    let heartbeat = warp::path!("heartbeat" / u64)
        .map(move |token: u64| {
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

            match env.update(&queue.lock().unwrap(),DURATION) {
                Ok(operations) => {
                    let image = renderer.render(operations);
                    keyboard_api.set_image(&image);
                }
                Err(error) => {
                    println!("{}", error);
                }
            }

            let end = Instant::now();
            let update_duration = end - begin;
            tokio::time::sleep(DURATION.saturating_sub(update_duration)).await;
        }
    });

    let paths = warp::post()
        .and(register
            .or(update)
            .or(heartbeat));
    warp::serve(paths).run(([127, 0, 0, 1], 3030)).await;
}