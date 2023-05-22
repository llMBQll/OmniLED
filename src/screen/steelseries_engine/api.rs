use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Mutex;
use lazy_static::lazy_static;
use serde_json::Value;
use ureq::{Agent, Error, Response};

pub fn update(data: &Vec<u8>) {
    API.lock().unwrap().update(data);
}

lazy_static!{
    static ref API: Mutex<Api> = Mutex::new(Api::new());
}

struct Api {
    agent: Agent,
    address: Option<String>,
    last_timestamp: Option<String>,
}

const GAME: &str = "RUST_STEELSERIES_OLED";
const GAME_DISPLAY_NAME: &str = "[Rust] Steelseries OLED";
const DEVELOPER: &str = "MBQ";
const TIMEOUT: u32 = 60000; // TODO: set this timeout from settings.lua ?

impl Api {
    pub fn new() -> Self {
        let mut api = Self {
            agent: Agent::new(),
            address: None,
            last_timestamp: None,
        };

        api
    }

    fn register(&mut self) {
        let metadata = serde_json::json!({
            "game": GAME,
            "game_display_name": GAME_DISPLAY_NAME,
            "developer": DEVELOPER,
            "deinitialize_timer_length_ms": TIMEOUT
        });
        self.game_metadata(serde_json::to_string(&metadata).unwrap().as_str());

        let handlers = serde_json::json!({
            "game": GAME,
            "golisp": "(handler \"UPDATE\" (lambda (data) (on-device 'screened show-image: (list-to-bytearray (image-data: (frame: data))))))"
        });
        self.load_golisp_handlers(serde_json::to_string(&handlers).unwrap().as_str());

        todo!("Register heartbeat event")
    }

    fn unregister(&mut self) {
        let remove_game = serde_json::json!({
            "game": GAME
        });

        self.remove_game(serde_json::to_string(&remove_game).unwrap().as_str());
    }

    pub fn update(&mut self, data: &Vec<u8>) {

    }

    fn game_metadata(&mut self, json: &str) {
        self.call("/game_metadata", json)
    }

    fn load_golisp_handlers(&mut self, json: &str) {
        self.call("/load_golisp_handlers", json)
    }

    fn game_event(&mut self, json: &str) {
        self.call("/game_event", json)
    }

    fn remove_game(&mut self, json: &str) {
        // TODO consider removing
        self.call("/remove_game", json)
    }

    fn game_heartbeat(&mut self, json: &str) {
        self.call("/game_heartbeat", json)
    }

    fn connect(&mut self, address: String, timestamp: String) {
        self.address = Some(address);
        self.last_timestamp = Some(timestamp);

        self.register();
    }

    fn call(&mut self, endpoint: &str, json: &str) {
        todo!("Move reassigning address to a separate function");
        if self.address.is_none() {
            let (address, timestamp) = match Self::get_address() {
                Some(value) => value,
                None => return
            };
            match &self.last_timestamp {
                Some(last_timestamp) => {
                    if timestamp == last_timestamp {
                        return;
                    }
                }
                None => {}
            }
            self.connect(address, timestamp);
        }

        let url = format!("http://{}{}", self.address.unwrap(), endpoint);
        let result = self.agent.post(url.as_str())
            .set("Content-Type", "application/json")
            .send_string(json);
        match result {
            Ok(response) => {
                todo!("Process response - this path probably can be left empty")
            }
            Err(error) => {
                match error {
                    Error::Status(_, _) => {}
                    Error::Transport(_) => {}
                }
                todo!("Process error")
            }
        }
    }

    fn get_address() -> Option<(String, String)> {
        // Missing directories are fatal errors
        let program_data = std::env::var("PROGRAMDATA")
            .expect("PROGRAMDATA env variable not found");
        let dir = format!("{}\\SteelSeries\\SteelSeries Engine 3", program_data);
        if !Path::new(&dir).is_dir() {
            panic!("{} doesn't exist", dir);
        }

        // Rest of the errors are non fatal, it is possible that Steelseries Engine is not yet ready
        let path = format!("{}\\coreProps.json", dir);
        let file = match File::open(path) {
            Ok(file) => file,
            Err(_) => return None
        };
        let reader = BufReader::new(file);
        let json: Value = match serde_json::from_reader(reader) {
            Ok(json) => json,
            Err(_) => return None
        };
        Some((
            json["address"]
                .as_str()
                .map(|address| {
                    String::from(address)
                })
                .unwrap(),
            json["timestamp"]
                .as_str()
                .map(|timestamp| {
                    String::from(timestamp)
                })
                .unwrap()
        ))
    }
}

impl Drop for Api {
    fn drop(&mut self) {
        self.unregister()
    }
}