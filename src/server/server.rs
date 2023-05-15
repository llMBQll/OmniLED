use log::{debug, error, info, LevelFilter, trace, warn};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use mlua::{chunk, Function, Lua};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::Instant;
use warp::{Filter};
use crate::applications::applications::{Applications, load_applications};
use crate::server::server::ApplicationMetadataReplyData::{Reason, Token};
use crate::server::update_handler::{load_update_handler, UpdateHandler};

pub struct Server {}

impl Server {
    pub fn new(lua: &Lua) {
        let applications = load_applications(lua);

        let register = warp::path!("register")
            .and(warp::body::json())
            .map({
                let applications = Arc::clone(&applications);
                move |metadata: ApplicationMetadata| {
                    match applications.lock().unwrap().register(&metadata.name) {
                        Some(token) => {
                            debug!("Registered application {} : {}", metadata.name, token);
                            let reply = warp::reply::json(&ApplicationMetadataReply {
                                metadata,
                                data: Token(token),
                            });
                            warp::reply::with_status(reply, warp::http::StatusCode::OK)
                        }
                        None => {
                            debug!("Failed to register {}", metadata.name);
                            let reply = warp::reply::json(&ApplicationMetadataReply {
                                metadata,
                                data: Reason(String::from("Application with the same name is already registered.")),
                            });
                            warp::reply::with_status(reply, warp::http::StatusCode::BAD_REQUEST)
                        }
                    }
                }
            });

        let heartbeat = warp::path!("heartbeat" / u64)
            .map({
                let applications = Arc::clone(&applications);
                move |token: u64| {
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
                }
            });

        let handler = load_update_handler(lua);
        let update = warp::path!("update" / u64)
            .and(warp::body::json())
            .map({
                let applications = Arc::clone(&applications);
                let handler = Arc::clone(&handler);
                move |token: u64, update: HashMap<String, serde_json::Value>| {
                    println!("{}", serde_json::to_string(&update).unwrap());
                    match applications.lock().unwrap().update(token) {
                        Some((name, timeout)) => {
                            handler.lock().unwrap().push((name, update));

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
                }
            });

        let paths = warp::post()
            .and(register)
            .or(update)
            .or(heartbeat);

        // Try to bind server to first available port and start accepting requests
        let mut port: u16 = 6969;
        let (address, server) = loop {
            match warp::serve(paths.clone()).try_bind_ephemeral(([127, 0, 0, 1], port)) {
                Ok((address, server)) => {
                    break (address.to_string(), server);
                }
                Err(_) => {
                    port += 1;
                }
            };
        };
        tokio::task::spawn(server); // TODO return server and start in main function

        // Make server address accessible from the Lua environment and also
        // from the filesystem for use in external applications
        const LOCALHOST: &str = "127.0.0.1";
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;

        let server = lua.create_table().unwrap();
        server.set("address", address.clone()).unwrap();
        server.set("ip", LOCALHOST).unwrap();
        server.set("port", port).unwrap();
        server.set("timestamp", timestamp).unwrap();
        lua.globals().set("SERVER", server).unwrap();

        let path = "server.json";
        let data = serde_json::json!({
            "address": address,
            "ip": LOCALHOST,
            "port": port,
            "timestamp": timestamp
        });
        std::fs::write(path, serde_json::to_string(&data).unwrap()).unwrap();
    }
}

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