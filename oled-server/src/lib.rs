use log::{error, Level};
use oled_api::types::{Event, EventReply, LogLevel, LogMessage};
use prost::bytes::Bytes;
use prost::Message;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use warp::reply::WithStatus;
use warp::Filter;

pub use warp::http::StatusCode;

pub trait RequestHandler {
    fn update(&mut self, event: Event) -> Result<(), (String, StatusCode)>;
    fn log(&mut self, level: Level, name: &str, message: &str) -> Result<(), (String, StatusCode)>;
}

pub struct Server {
    pub address: String,
    pub ip: String,
    pub port: u16,
    pub timestamp: u64,
    server: Pin<Box<dyn Future<Output = ()> + Send>>,
}

impl Server {
    pub fn bind<T: RequestHandler + Send + Sync + 'static>(
        implementation: T,
        mut port: u16,
        strict: bool,
    ) -> Self {
        let implementation = Arc::new(Mutex::new(implementation));

        let update = warp::path!("update").and(warp::body::bytes()).map({
            let implementation = Arc::clone(&implementation);
            move |bytes: Bytes| {
                let event = match Event::decode(bytes) {
                    Ok(event) => event,
                    Err(err) => {
                        return Self::reply(Err((err.to_string(), StatusCode::BAD_REQUEST)));
                    }
                };

                let result = implementation.lock().unwrap().update(event);
                Self::reply(result)
            }
        });

        let log = warp::path!("log").and(warp::body::bytes()).map({
            let implementation = Arc::clone(&implementation);
            move |bytes: Bytes| {
                let log = match LogMessage::decode(bytes) {
                    Ok(event) => event,
                    Err(err) => {
                        return Self::reply(Err((err.to_string(), StatusCode::BAD_REQUEST)));
                    }
                };
                let level = match oled_api::types::LogLevel::try_from(log.severity) {
                    Ok(LogLevel::Error) => Level::Error,
                    Ok(LogLevel::Warn) => Level::Warn,
                    Ok(LogLevel::Info) => Level::Info,
                    Ok(LogLevel::Debug) => Level::Debug,
                    Ok(LogLevel::Trace) => Level::Trace,
                    Err(err) => {
                        return Self::reply(Err((err.to_string(), StatusCode::BAD_REQUEST)));
                    }
                };

                let result = implementation
                    .lock()
                    .unwrap()
                    .log(level, &log.name, &log.message);
                Self::reply(result)
            }
        });

        let paths = warp::post().and(update).or(log);

        // Try to bind to a set port, if allowed try binding to next available port until it succeeds
        let (address, server) = loop {
            match warp::serve(paths.clone()).try_bind_ephemeral(([127, 0, 0, 1], port)) {
                Ok((address, server)) => {
                    break (address, server);
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

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        Self {
            address: address.to_string(),
            ip: address.ip().to_string(),
            port: address.port(),
            timestamp,
            server: Box::pin(server),
        }
    }

    pub async fn run(self) {
        self.server.await
    }

    fn reply(result: Result<(), (String, StatusCode)>) -> WithStatus<Vec<u8>> {
        let (reply, status_code) = match result {
            Ok(_) => (EventReply { error: None }, StatusCode::OK),
            Err((error, status_code)) => (EventReply { error: Some(error) }, status_code),
        };
        let bytes = reply.encode_to_vec();
        warp::reply::with_status(bytes, status_code)
    }
}
