use log::{log, Level};
use mlua::{Lua, LuaSerdeExt};
use oled_api::types::Event;
use oled_server::{RequestHandler, Server, StatusCode};
use std::sync::{Arc, Mutex};

use crate::constants::constants::Constants;
use crate::events;
use crate::events::event_queue::EventQueue;
use crate::settings::settings::Settings;

pub struct PluginServer {}

impl PluginServer {
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
}

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
        if !is_valid_event_name(&event.name) {
            return Err((String::from("Invalid event name"), StatusCode::BAD_REQUEST));
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

pub fn is_valid_event_name(name: &str) -> bool {
    if name.len() == 0 {
        return false;
    }

    let mut chars = name.chars();

    let first = chars.next().unwrap();
    if first != '_' && (first < 'A' || first > 'Z') {
        return false;
    }

    for c in chars {
        if c != '_' && (c < 'A' || c > 'Z') && (c < '0' || c > '9') {
            return false;
        }
    }

    true
}
