use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};
use mlua::Lua;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use warp::{Filter};
use crate::server::update_handler::load_update_handler;

pub struct Server {}

impl Server {
    pub fn new(lua: &Lua) {
        let state = Arc::new(Mutex::new(State::new()));

        let register = warp::path!("register")
            .and(warp::body::json())
            .map({
                let state = Arc::clone(&state);
                move |data: RegisterData| {
                    let token = state.lock().unwrap().add(data.name);

                    let reply = warp::reply::json(&RegisterReply { token });
                    warp::reply::with_status(reply, warp::http::StatusCode::OK)
                }
            });

        let handler = load_update_handler(lua);
        let update = warp::path!("update" / u64)
            .and(warp::body::json())
            .map({
                let state = Arc::clone(&state);
                let handler = Arc::clone(&handler);
                move |token: u64, update_data: UpdateData| {
                    match state.lock().unwrap().get(token) {
                        Some(name) => {
                            handler.lock().unwrap().push((name.clone(), update_data.fields));

                            let reply = warp::reply::json(&UpdateReply { error: None });
                            warp::reply::with_status(reply, warp::http::StatusCode::OK)
                        }
                        None => {
                            let reply = warp::reply::json(&UpdateReply {
                                error: Some(String::from("Application not registered."))
                            });
                            warp::reply::with_status(reply, warp::http::StatusCode::BAD_REQUEST)
                        }
                    }
                }
            });

        let paths = warp::post()
            .and(register)
            .or(update);

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

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RegisterData {
    name: String,
}

#[derive(Serialize)]
struct RegisterReply {
    token: u64,
}

#[derive(Deserialize)]
struct UpdateData {
    #[serde(flatten)]
    fields: HashMap<String, serde_json::Value>,
}

#[derive(Serialize)]
struct UpdateReply {
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

struct State {
    names: HashMap<u64, String>,
}

impl State {
    pub fn new() -> Self {
        Self {
            names: HashMap::new(),
        }
    }

    pub fn add(&mut self, name: String) -> u64 {
        let mut s = DefaultHasher::new();
        name.hash(&mut s);
        let token = s.finish();

        self.names.insert(token, name);

        token
    }

    pub fn get(&self, token: u64) -> Option<&String> {
        self.names.get(&token)
    }
}