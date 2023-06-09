use mlua::Lua;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use warp::Filter;

use crate::server::update_handler::load_update_handler;

pub struct Server {}

impl Server {
    pub fn new(lua: &Lua) {
        let handler = load_update_handler(lua);
        let update = warp::path!("update")
            .and(warp::body::json())
            .map(move |update_data: UpdateData| {
                handler.lock().unwrap().push((update_data.name, update_data.fields));

                let reply = warp::reply::json(&UpdateReply { error: None });
                warp::reply::with_status(reply, warp::http::StatusCode::OK)
            });

        // Try to bind server to first available port and start accepting requests
        let mut port: u16 = 6969;
        let (address, server) = loop {
            match warp::serve(update.clone()).try_bind_ephemeral(([127, 0, 0, 1], port)) {
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
struct UpdateData {
    name: String,
    fields: HashMap<String, serde_json::Value>,
}

#[derive(Serialize)]
struct UpdateReply {
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}