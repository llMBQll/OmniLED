use clap::Parser;
use oled_api::{EventData, EventResponse, LogMessage, LogResponse, Plugin};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::process::{Command, Stdio};
use tonic::transport::Server;
use tonic::{Code, Request, Response, Status};

#[tokio::main]
async fn main() {
    let options = Options::parse();

    let port = options.port;
    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
    tokio::task::spawn(
        Server::builder()
            .add_service(oled_api::plugin_server::PluginServer::new(RequestPrinter))
            .serve(address),
    );

    println!("Bound to {}", address.to_string());

    let args: Vec<String> = options
        .extras
        .into_iter()
        .map(|arg| {
            if arg == options.address_map {
                address.to_string()
            } else {
                arg
            }
        })
        .collect();
    let mut command = Command::new(&options.path);
    command
        .args(&args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    println!("Running {{{:?}}}\n\n", command);

    let mut process = command.spawn().unwrap();

    process.wait().unwrap();
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Options {
    #[clap(short = 'p', long, default_value_t = 6969)]
    port: u16,

    #[clap(short = 'P', long)]
    path: String,

    /// Replace this string with server address in subject arguments
    #[clap(short, long, default_value = "_@@_")]
    address_map: String,

    #[clap(last = true, allow_hyphen_values = true)]
    extras: Vec<String>,
}

struct RequestPrinter;

#[tonic::async_trait]
impl oled_api::plugin_server::Plugin for RequestPrinter {
    async fn event(&self, request: Request<EventData>) -> Result<Response<EventResponse>, Status> {
        let event = request.get_ref();

        if !Plugin::is_valid_identifier(&event.name) {
            return Err(Status::new(Code::InvalidArgument, "Invalid event name"));
        }

        println!("{:?}", event);

        Ok(Response::new(EventResponse {}))
    }

    async fn log(&self, request: Request<LogMessage>) -> Result<Response<LogResponse>, Status> {
        let log = request.get_ref();

        let level = match Plugin::log_level_from_integer(log.severity) {
            Ok(level) => level,
            Err(err) => return Err(Status::new(Code::InvalidArgument, err.to_string())),
        };

        println!("[{}] plugin::{} - {}", level, log.name, log.message);

        Ok(Response::new(LogResponse {}))
    }
}
