use log::{error, log};
use mlua::{Lua, UserData, UserDataFields};
use oled_api::{EventData, EventResponse, LogData, LogResponse, Plugin};
use serde::Serialize;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::net::TcpListener;
use tokio_stream::StreamExt;
use tonic::transport::Server;
use tonic::{Code, Request, Response, Status, Streaming};

use crate::common::user_data::UserDataRef;
use crate::constants::constants::Constants;
use crate::events;
use crate::events::event_queue::EventQueue;
use crate::settings::settings::Settings;

pub struct PluginServer {
    event_queue: Arc<Mutex<EventQueue>>,
    log_level_filter: log::LevelFilter,
}

impl PluginServer {
    pub async fn load(lua: &Lua) {
        let settings = UserDataRef::<Settings>::load(lua);

        let port: u16 = settings.get().server_port;
        let listener = TcpListener::bind(format!("127.0.0.1:{port}"))
            .await
            .unwrap();
        let address = listener.local_addr().unwrap();
        let bound_port = address.port();

        let log_level_filter = settings.get().log_level.into();

        tokio::task::spawn(
            Server::builder()
                .add_service(oled_api::plugin_server::PluginServer::new(Self::new(log_level_filter)))
                .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener)),
        );

        let address = format!("127.0.0.1:{bound_port}");
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|err| {
                error!("Failed to get unix epoch time: {}", err);
                Duration::ZERO
            })
            .as_millis() as u64;
        let info = ServerInfo {
            address,
            ip: String::from("127.0.0.1"),
            port: bound_port,
            timestamp,
        };

        std::fs::write(
            Constants::data_dir().join("server.json"),
            serde_json::to_string_pretty(&info).unwrap(),
        )
        .unwrap();

        lua.globals().set("SERVER", info).unwrap();
    }

    fn new(log_level_filter: log::LevelFilter) -> Self {
        Self {
            event_queue: EventQueue::instance(),
            log_level_filter,
        }
    }
}

#[derive(Clone, Serialize)]
struct ServerInfo {
    address: String,
    ip: String,
    port: u16,
    timestamp: u64,
}

impl UserData for ServerInfo {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("address", |_, info| Ok(info.address.clone()));
        fields.add_field_method_get("ip", |_, info| Ok(info.ip.clone()));
        fields.add_field_method_get("port", |_, info| Ok(info.port));
        fields.add_field_method_get("timestamp", |_, info| Ok(info.timestamp));
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

        let fields = match fields {
            Some(fields) => fields,
            None => std::unreachable!("This has type Option<Table> due the code generator, field is required in proto definition")
        };

        self.event_queue
            .lock()
            .unwrap()
            .push(events::event_queue::Event::Application((
                name,
                fields.items,
            )));

        Ok(Response::new(EventResponse {}))
    }

    async fn log(
        &self,
        request: Request<Streaming<LogData>>,
    ) -> Result<Response<LogResponse>, Status> {
        let mut in_stream = request.into_inner();

        tokio::spawn(async move {
            while let Some(result) = in_stream.next().await {
                match result {
                    Ok(data) => {
                        log!(target: &data.location, data.log_level().into(), "{}", data.message);
                    }
                    Err(err) => {
                        panic!("Connection closing is not yet handled properly: {}", err);
                    }
                }
            }
        });

        let mut response = LogResponse::default();
        response.set_log_level_filter(self.log_level_filter.into());
        Ok(Response::new(response))
    }
}
