use log::log;
use mlua::{Lua, LuaSerdeExt};
use oled_api::{EventData, EventResponse, LogMessage, LogResponse, Plugin};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tonic::transport::Server;
use tonic::{Code, Request, Response, Status};

use crate::common::user_data::UserDataRef;
use crate::constants::constants::Constants;
use crate::events;
use crate::events::event_queue::EventQueue;
use crate::settings::settings::Settings;

pub struct PluginServer {
    event_queue: Arc<Mutex<EventQueue>>,
}

impl PluginServer {
    pub async fn load(lua: &Lua) {
        let settings = UserDataRef::<Settings>::load(lua);
        let port: u16 = settings.get().server_port;

        let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
        tokio::task::spawn(
            Server::builder()
                .add_service(oled_api::plugin_server::PluginServer::new(Self::new()))
                .serve(address),
        );

        let address = format!("127.0.0.1:{port}");
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        let info = serde_json::json!({
            "address": address,
            "ip": "127.0.0.1",
            "port": port,
            "timestamp": timestamp,
        });

        lua.globals()
            .set("SERVER", lua.to_value(&info).unwrap())
            .unwrap();

        std::fs::write(
            Constants::root_dir().join("server.json"),
            serde_json::to_string_pretty(&info).unwrap(),
        )
        .unwrap();
    }

    fn new() -> Self {
        Self {
            event_queue: EventQueue::instance(),
        }
    }
}

#[tonic::async_trait]
impl oled_api::plugin_server::Plugin for PluginServer {
    async fn event(
        &self,
        mut request: Request<EventData>,
    ) -> Result<Response<EventResponse>, Status> {
        let event = request.get_mut();

        if !Plugin::is_valid_identifier(&event.name) {
            return Err(Status::new(Code::InvalidArgument, "Invalid event name"));
        }

        let mut name = String::new();
        std::mem::swap(&mut name, &mut event.name);

        let mut fields = None;
        std::mem::swap(&mut fields, &mut event.fields);

        self.event_queue
            .lock()
            .unwrap()
            .push(events::event_queue::Event::Application((
                name,
                fields.unwrap().items,
            )));

        Ok(Response::new(EventResponse {}))
    }

    async fn log(&self, request: Request<LogMessage>) -> Result<Response<LogResponse>, Status> {
        let log = request.get_ref();

        let level = match Plugin::log_level_from_integer(log.severity) {
            Ok(level) => level,
            Err(err) => return Err(Status::new(Code::InvalidArgument, err.to_string())),
        };
        let target = format!("plugin::{}", log.name);

        log!(target: &target, level, "{}", log.message);

        Ok(Response::new(LogResponse {}))
    }
}
