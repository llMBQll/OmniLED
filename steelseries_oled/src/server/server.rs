use log::{log, Level};
use mlua::{Lua, LuaSerdeExt};
use oled_api::types::Event;
use oled_server::{RequestHandler, Server, StatusCode};
use std::sync::{Arc, Mutex};

use crate::constants::constants::Constants;
use crate::events;
use crate::events::event_queue::EventQueue;
use crate::settings::settings::Settings;

struct PluginRequestHandler {
    event_queue: Arc<Mutex<EventQueue>>,
}

impl PluginRequestHandler {
    pub fn new() -> Self {
        Self {
            event_queue: EventQueue::instance(),
        }
    }
}

impl RequestHandler for PluginRequestHandler {
    fn update(&mut self, event: Event) -> Result<(), (String, StatusCode)> {
        if is_alpha_uppercase(&event.name) {
            return Err((
                String::from("Event name is not alpha uppercase"),
                StatusCode::BAD_REQUEST,
            ));
        }

        self.event_queue
            .lock()
            .unwrap()
            .push(events::event_queue::Event::Application((
                event.name,
                event.fields.unwrap().items,
            )));

        Ok(())
    }

    fn log(&mut self, level: Level, name: &str, message: &str) -> Result<(), (String, StatusCode)> {
        let target = format!("plugin::{}", name);
        log!(target: &target, level, "{}", message);

        Ok(())
    }
}

fn is_alpha_uppercase(name: &str) -> bool {
    for c in name.chars() {
        if c < 'A' || c > 'Z' {
            return false;
        }
    }
    true
}

pub fn load(lua: &Lua) {
    let implementation = PluginRequestHandler::new();
    let port: u16 = Settings::get().server_port;
    let strict: bool = Settings::get().server_port_strict;

    let server = Server::bind(implementation, port, strict);

    let info = serde_json::json!({
        "address": server.address,
        "ip": server.ip,
        "port": server.port,
        "timestamp": server.timestamp,
    });

    tokio::task::spawn(server.run());

    lua.globals()
        .set("SERVER", lua.to_value(&info).unwrap())
        .unwrap();

    std::fs::write(
        Constants::root_dir().join("server.json"),
        serde_json::to_string_pretty(&info).unwrap(),
    )
    .unwrap();
}
