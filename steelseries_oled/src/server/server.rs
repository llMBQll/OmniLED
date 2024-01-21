use api::types::{Event, EventReply};
use log::error;
use mlua::{Lua, LuaSerdeExt};
use prost::bytes::Bytes;
use prost::Message;
use std::time::{SystemTime, UNIX_EPOCH};
use warp::http::StatusCode;
use warp::reply::WithStatus;
use warp::Filter;

use crate::constants::constants::Constants;
use crate::events;
use crate::events::event_queue::EventQueue;
use crate::settings::settings::Settings;

pub struct Server {}

impl Server {
    pub fn load(lua: &Lua) {
        let event_queue = EventQueue::instance();
        let update = warp::path!("update")
            .and(warp::body::bytes())
            .map(move |bytes: Bytes| {
                let event = match Event::decode(&*bytes) {
                    Ok(event) => event,
                    Err(err) => {
                        return Self::reply(Some(err.to_string()), StatusCode::BAD_REQUEST);
                    }
                };

                event_queue
                    .lock()
                    .unwrap()
                    .push(events::event_queue::Event::Application((
                        event.name,
                        event.fields.unwrap().items,
                    )));

                Self::reply(None, StatusCode::OK)
            });

        // Try to bind to a set port, if allowed try binding to next available port until it succeeds
        let mut port: u16 = Settings::get().server_port;
        let strict: bool = Settings::get().server_port_strict;
        let (address, server) = loop {
            match warp::serve(update.clone()).try_bind_ephemeral(([127, 0, 0, 1], port)) {
                Ok((address, server)) => {
                    break (address.to_string(), server);
                }
                Err(err) => {
                    if strict {
                        error!("Failed to open a server on port {}: {}", port, err);
                        panic!("Failed to start a server");
                    }

                    port += 1;
                }
            };
        };
        tokio::task::spawn(server); // TODO return server and start in main function

        // Make server address accessible from the Lua environment and also
        // from the filesystem for use in external applications
        const LOCALHOST: &str = "127.0.0.1";
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        let data = serde_json::json!({
            "address": address,
            "ip": LOCALHOST,
            "port": port,
            "timestamp": timestamp
        });

        lua.globals()
            .set("SERVER", lua.to_value(&data).unwrap())
            .unwrap();
        std::fs::write(
            Constants::root_dir().join("server.json"),
            serde_json::to_string_pretty(&data).unwrap(),
        )
        .unwrap();
    }

    fn reply(error: Option<String>, status_code: StatusCode) -> WithStatus<Vec<u8>> {
        let reply = EventReply { error };
        let bytes = reply.encode_to_vec();
        warp::reply::with_status(bytes, status_code)
    }
}
