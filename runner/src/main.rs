use crate::plugin::Plugin;

mod plugin;

use std::{env, time};
use std::net::{TcpStream};
use std::io::Write;
use std::process;
use std::thread;
use common_rs::interface::StatusCode;

static mut STREAM: Option<TcpStream> = None;

fn main() {
    let args: Vec<String> = env::args().collect();
    let addr = &args[1];
    let path = &args[2];

    match TcpStream::connect(addr) {
        Ok(stream) => { unsafe { STREAM = Some(stream); } }
        Err(_) => {
            eprintln!("Failed to connect to {}", addr);
            process::exit(1);
        }
    }

    let mut plugin = Plugin::new(path).unwrap();
    plugin.run(on_update);

    let duration = time::Duration::from_millis(1234);
    thread::sleep(duration);

    let res = match plugin.stop() {
        Some(res) => res,
        None => StatusCode::Ok
    };
    match res {
        StatusCode::Ok => process::exit(0),
        StatusCode::Error => process::exit(1)
    }
}

extern fn on_update(str: *const u8, len: u32) -> StatusCode {
    let mut stream = unsafe { STREAM.as_ref().unwrap() };

    let buf = unsafe { std::slice::from_raw_parts(str, len as usize) };
    match stream.write(buf) {
        Ok(_) => StatusCode::Ok,
        Err(_) => StatusCode::Error
    }
}