use log::error;
use mlua::{Lua, UserData, UserDataFields};
use oled_api::{EventData, EventResponse, Plugin, RequestDirectoryData, RequestDirectoryResponse};
use serde::Serialize;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::net::TcpListener;
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

        let listener = TcpListener::bind(format!("127.0.0.1:{port}"))
            .await
            .unwrap();
        let address = listener.local_addr().unwrap();
        let bound_port = address.port();

        tokio::task::spawn(
            Server::builder()
                .add_service(oled_api::plugin_server::PluginServer::new(Self::new()))
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

    fn new() -> Self {
        Self {
            event_queue: EventQueue::instance(),
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

    async fn request_directory(
        &self,
        request: Request<RequestDirectoryData>,
    ) -> Result<Response<RequestDirectoryResponse>, Status> {
        let data = request.get_ref();

        if !Plugin::is_valid_identifier(&data.name) {
            return Err(Status::new(Code::InvalidArgument, "Invalid event name"));
        }

        let path = Constants::data_dir().join(data.name.to_ascii_lowercase());
        if !path.exists() {
            if let Err(err) = tokio::fs::create_dir_all(&path).await {
                return Err(Status::new(Code::Internal, err.to_string()));
            }
        }

        Ok(Response::new(RequestDirectoryResponse {
            directory: path.to_string_lossy().to_string(),
        }))
    }
}
