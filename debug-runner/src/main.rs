use clap::Parser;
use log::Level;
use oled_api::types::Event;
use oled_server::{RequestHandler, Server, StatusCode};
use std::process::{Command, Stdio};

#[tokio::main]
async fn main() {
    let options = Options::parse();

    let server = Server::bind(RequestPrinter, options.port, options.strict);
    println!("Bound to {}", server.address);

    let args: Vec<String> = options
        .extras
        .into_iter()
        .map(|arg| {
            if arg == options.address_map {
                server.address.clone()
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

    tokio::task::spawn(server.run());
    process.wait().unwrap();
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Options {
    #[clap(short = 'p', long, default_value_t = 6969)]
    port: u16,

    #[clap(short, long)]
    strict: bool,

    #[clap(short = 'P', long)]
    path: String,

    /// Replace this string with server address in subject arguments
    #[clap(short, long, default_value = "_@@_")]
    address_map: String,

    #[clap(last = true, allow_hyphen_values = true)]
    extras: Vec<String>,
}

struct RequestPrinter;

impl RequestHandler for RequestPrinter {
    fn update(&mut self, event: Event) -> Result<(), (String, StatusCode)> {
        println!("{:?}", event);

        Ok(())
    }

    fn log(&mut self, level: Level, name: &str, message: &str) -> Result<(), (String, StatusCode)> {
        println!("[{}] plugin::{} - {}", level, name, message);

        Ok(())
    }
}
