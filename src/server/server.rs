use mlua::Lua;
use std::time::{SystemTime, UNIX_EPOCH};
use warp::Filter;

pub struct Server {}

impl Server {
    pub fn new(lua: &Lua) {
        let xd = warp::path!("XD").map(|| {
            warp::reply::with_status("XD", warp::http::StatusCode::OK)
        });

        let paths = warp::post().and(xd);

        // Try to bind server to first available port
        let mut port: u16 = 6969;
        let (address, server) = loop {
            match warp::serve(paths.clone()).try_bind_ephemeral(([127, 0, 0, 1], port)) {
                Ok((address, server)) => {
                    break (address.to_string(), server);
                }
                Err(_) => {
                    port += 1;
                }
            };
        };
        tokio::task::spawn(server);

        let server = lua.create_table().unwrap();
        server.set("address", format!("{}:{}", "127.0.0.1", port)).unwrap();
        server.set("ip", "127.0.0.1").unwrap();
        server.set("port", port).unwrap();
        lua.globals().set("SERVER", server).unwrap();

        // Write server address in case some non-registered application wants to send requests
        let path = "server.json";
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
        let data = serde_json::json!({
            "address": address,
            "ip": "127.0.0.1",
            "port": port,
            "timestamp": timestamp
        });
        std::fs::write(path, serde_json::to_string(&data).unwrap()).unwrap();
    }
}
