// use std::{thread, time};
// use std::fs::File;
//
// use crate::lisp_handler::lisp_handler::LispHandler;
// use crate::model::display::Display;
// use crate::model::position::Position;
// use crate::plugin::plugin::Plugin;
// use crate::renderer::renderer::Renderer;
// use crate::steelseries_api::SteelSeriesAPI;
//
// mod lisp_handler;
// mod renderer;
// mod plugin;
// mod model;
// mod steelseries_api;
//
// const WIDTH: usize = 128;
// const HEIGHT: usize = 40;
//
// const HANDLER: &str = r#"(handler \"UPDATE\" (lambda (data) (on-device 'screened show-image: (list-to-bytearray (image-data: (frame: data)))))) (add-event-zone-use-with-specifier \"CLOCK_UPDATE\" \"one\" 'screened)"#;
//
// fn path(name: &str) -> String {
//     #[cfg(debug_assertions)]
//     let str = format!("target\\debug\\{}", name);
//
//     #[cfg(not(debug_assertions))]
//     let str = format!("target\\release\\{}", name);
//
//     println!("Loading: {}", str);
//
//     str
// }
//
// fn main() {
//     // let mut file = File::open("displays.json").unwrap();
//     // file.write_all(serde_json::to_string_pretty(&displays).unwrap().as_ref()).unwrap();
//
//     let mut file = File::open("displays.json").unwrap();
//     let displays: Vec<Display> = serde_json::from_reader(&mut file).unwrap();
//
//     let spotify_plugin = Plugin::new(&path("spotify.dll")).expect("Failed to load");
//     let clock_plugin = Plugin::new(&path("clock.dll")).expect("Failed to load");
//     let audio_plugin = Plugin::new(&path("audio.dll")).expect("Failed to load");
//     let _ = clock_plugin.types();
//
//     let mut handler = LispHandler::new();
//     let mut renderer = Renderer::new(HEIGHT, WIDTH);
//     let mut api = SteelSeriesAPI::new();
//
//     match api.remove_game(r#"{"game":"RUST_STEELSERIES_OLED"}"#) {
//         _ => { }
//     }
//     api.game_metadata(r#"{"game":"RUST_STEELSERIES_OLED", "game_display_name":"[Rust] Steelseries OLED", "developer":"MBQ"}"#).expect("/game_metadata");
//     api.load_lisp_handlers(format!(r#"{{"game":"RUST_STEELSERIES_OLED", "golisp":"{}"}}"#, HANDLER).as_str()).expect("/load_lisp_handlers");
//
//     handler.register(displays).expect("Register failed");
//
//     let duration = time::Duration::from_millis(50);
//     let mut last_update = time::Instant::now();
//     const MAX_UPDATE_INTERVAL: time::Duration = time::Duration::from_secs(10);
//     loop {
//         let update_begin = time::Instant::now();
//
//         let mut plugins = Vec::new();
//         plugins.push((audio_plugin.name(), audio_plugin.update()));
//         plugins.push((clock_plugin.name(), clock_plugin.update()));
//         plugins.push((spotify_plugin.name(), spotify_plugin.update()));
//         let results = handler.update(&plugins, duration);
//
//         match results {
//             Ok(results) => {
//                 match results.len() {
//                     0 => {
//                         if update_begin.saturating_duration_since(last_update) > MAX_UPDATE_INTERVAL {
//                             last_update = update_begin;
//                             api.game_heartbeat(r#"{"game": "RUST_STEELSERIES_OLED"}"#).expect("/game_heartbeat");
//                         }
//                     }
//                     _ => {
//                         let update = serde_json::json!({
//                             "game": "RUST_STEELSERIES_OLED",
//                             "event": "UPDATE",
//                             "data": {
//                                 "value": 0,
//                                 "frame": {
//                                     "image-data": renderer.render(results)
//                                 }
//                             }
//                         });
//                         api.game_event(serde_json::to_string(&update).unwrap().as_str()).expect("/game_event");
//                     }
//                 }
//             }
//             Err(err) => {
//                 println!("{}", err);
//             }
//         }
//
//         let update_end = time::Instant::now();
//         let update_duration = update_end - update_begin;
//         thread::sleep(duration.saturating_sub(update_duration));
//     }
// }
//
// use std::io::{Cursor, Read};
// use std::net::{TcpListener, TcpStream};
// use std::{thread, time};
// use std::iter::Map;
// use std::string::FromUtf8Error;
// use std::sync::{Arc, Mutex};
// use byteorder::{LittleEndian, ReadBytesExt};
// use crate::plugin::plugin::Plugin;
//
// mod plugin;
//
// const SLEEP_DURATION: time::Duration = time::Duration::from_secs(60);
//
// fn main() {
//     let addr = String::from("localhost:1337");
//     let listener = TcpListener::bind(&addr).unwrap();
//
//     let queue = Arc::new(Mutex::new(MessageQueue::new()));
//     let handle = std::thread::spawn(move || {
//         for stream in listener.incoming() {
//             match stream {
//                 Ok(stream) => { handle_stream(stream, queue.clone()); }
//                 Err(err) => { eprintln!("{}", err); }
//             }
//         }
//     });
//
//     let names = ["audio", "clock"];
//     let mut plugins = Vec::with_capacity(names.len());
//     for name in names {
//         match Plugin::new(String::from(name), &addr) {
//             Ok(plugin) => {
//                 println!("Loaded '{}' plugin", name);
//                 plugins.push(plugin);
//             },
//             Err(err) => {
//                 println!("Error while loading '{}' plugin: {}", name, err);
//             }
//         };
//     }
//
//     thread::sleep(SLEEP_DURATION);
//
//     for mut plugin in plugins {
//         match plugin.stop() {
//             Ok(exit_code) => {
//                 println!("Plugin '{}' exited with code [{}]", plugin.name(), exit_code);
//             }
//             Err(err) => {
//                 println!("Plugin '{}' had an error while exiting: {}", plugin.name(), err);
//             }
//         }
//     }
//
//     handle.join().unwrap();
// }
//
// fn handle_stream(mut stream: TcpStream, queue: Arc<Mutex<MessageQueue>>) {
//     let mut msg_buf = [0 as u8; 1024];
//     loop {
//         let len = stream.read_u32::<LittleEndian>().unwrap() as usize;
//
//         let mut message = Vec::<u8>::with_capacity(len);
//         let mut bytes_read = 0;
//         while bytes_read != len {
//             // let n = stream.read_buf_exact()
//         }
//
//         // println!("read");
//         // let n = stream.read(&mut msg_buf).unwrap();
//         // if n == 0 {
//         //     println!("n == 0");
//         //     break;
//         // }
//         // match String::from_utf8(msg_buf.to_vec()) {
//         //     Ok(msg) => {
//         //         let mut queue = queue.lock().unwrap();
//         //         queue.push(msg);
//         //     }
//         //     Err(err) => {
//         //         println!("Invalid utf-8: {}", err);
//         //     }
//         // }
//
//     }
// }
//
// struct MessageQueue {
//     messages: Vec<String>,
// }
//
// impl MessageQueue {
//     pub fn new() -> Self {
//         Self {
//             messages: Vec::new(),
//         }
//     }
//
//     pub fn push(&mut self, msg: String) {
//         self.messages.push(msg);
//     }
//
//     pub fn take(&mut self) -> Vec<String> {
//         let mut tmp = Vec::new();
//         std::mem::swap(&mut self.messages, &mut tmp);
//         tmp
//     }
// }

use std::collections::hash_map::{DefaultHasher, Entry};
use std::collections::HashMap;
use std::fs::File;
use std::hash::{Hash, Hasher};
use warp::Filter;
use serde::{Deserialize, Serialize};
use tokio::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use crate::ApplicationMetadataReplyData::{Reason, Token};

use crate::lisp_handler::lisp_handler::LispHandler;
use crate::keyboard_api::KeyboardAPI;
use crate::renderer::renderer::Renderer;
use crate::model::display::Display;

mod keyboard_api;
mod lisp_handler;
mod model;
mod renderer;

// use crate::model::position::Position;
// use crate::plugin::plugin::Plugin;
// use crate::steelseries_api::SteelSeriesAPI;
//
// mod plugin;
// mod steelseries_api;

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

    let mut _env = setup_env();
    let mut _renderer = setup_renderer();
    let mut _keyboard_api = setup_keyboard_api();
    // let _endpoints = setup_endpoints();

    run_server().await;
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

async fn run_server() {
    let applications = Arc::new(Mutex::new(Applications::new()));


    let applications_c = Arc::clone(&applications);

    let register = warp::post()
        .and(warp::path("register"))
        .and(warp::body::json())
        .map(move |metadata: ApplicationMetadata| {
            match applications_c.lock().unwrap().register(&metadata.name) {
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

    let applications_c = Arc::clone(&applications);
    let update = warp::path!("update" / u64)
        .and(warp::body::json())
        .map(move |token: u64, _update: serde_json::Value| {
            match applications_c.lock().unwrap().update(token) {
                Some((_name, timeout)) => {
                    // TODO process update

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

    let applications_c = Arc::clone(&applications);
    let heartbeat = warp::path!("heartbeat" / u64)
        .map(move |token: u64| {
            match applications_c.lock().unwrap().update(token) {
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

    let paths = warp::post().and(
        register
            .or(update)
            .or(heartbeat)
    );

    warp::serve(paths).run(([127, 0, 0, 1], 3030)).await;
}