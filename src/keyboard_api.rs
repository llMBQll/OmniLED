use std::fs::File;
use std::io::BufReader;
use std::{thread, time};
use std::path::Path;
use serde_json::Value;
use ureq::{Agent, Response};


pub struct KeyboardAPI {
    agent: Agent,
    address: Option<String>,
}

const GAME: &str = "RUST_STEELSERIES_OLED";
const GAME_DISPLAY_NAME: &str = "[Rust] Steelseries OLED";
const DEVELOPER: &str = "MBQ";
const TIMEOUT: u32 = 60000;

impl KeyboardAPI {
    pub fn new() -> Self {
        let mut api = KeyboardAPI {
            agent: Agent::new(),
            address: match Self::get_address() {
                Ok(address) => Some(address),
                Err(_) => None
            },
        };

        let metadata = serde_json::json!({
            "game": GAME,
            "game_display_name": GAME_DISPLAY_NAME,
            "developer": DEVELOPER,
            "deinitialize_timer_length_ms": TIMEOUT
        });
        api.game_metadata(serde_json::to_string(&metadata).unwrap().as_str())
            .expect("Failed to register application with Steelseries API");

        let handlers = serde_json::json!({
            "game": GAME,
            "golisp": "(handler \"UPDATE\" (lambda (data) (on-device 'screened show-image: (list-to-bytearray (image-data: (frame: data))))))"
        });
        api.load_golisp_handlers(serde_json::to_string(&handlers).unwrap().as_str())
            .expect("Failed to register handlers with Steelseries API");

        // TODO register heartbeat event

        api
    }

    pub fn set_image(&mut self, image: &Vec<u8>) {
        let json = serde_json::json!({
            "game": GAME,
            "event": "UPDATE",
            "data": {
                "value": 0,
                "frame": {
                    "image-data": image
                }
            }
        });

        self.game_event(serde_json::to_string(&json).unwrap().as_str()).unwrap();
    }

    fn get_address() -> Result<String, SilentError> {
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
            Err(_) => return Err(SilentError {})
        };
        let reader = BufReader::new(file);
        let json: Value = match serde_json::from_reader(reader) {
            Ok(json) => json,
            Err(_) => return Err(SilentError {})
        };
        json["address"].as_str()
            .map(|address| {
                String::from(address)
            })
            .ok_or(SilentError {})
    }

    fn wait_for_address(&mut self) {
        if self.address.is_some() {
            return;
        }
        loop {
            match Self::get_address() {
                Ok(address) => self.address = Some(address),
                Err(_) => {
                    // Ignore error, wait for Steelseries Engine
                    println!("Waiting for Steelseries Engine");
                    thread::sleep(time::Duration::from_secs(5));
                }
            }
        }
    }

    fn call(&mut self, endpoint: &str, json: &str) -> Result<Response, ureq::Error> {
        self.wait_for_address();
        let address: &String = self.address.as_ref().unwrap();
        let url = format!("http://{}{}", address, endpoint);
        self.agent.post(url.as_str())
            .set("Content-Type", "application/json")
            .send_string(json)
    }

    fn game_metadata(&mut self, json: &str) -> Result<Response, ureq::Error> {
        self.call("/game_metadata", json)
    }

    fn load_golisp_handlers(&mut self, json: &str) -> Result<Response, ureq::Error> {
        self.call("/load_golisp_handlers", json)
    }

    fn game_event(&mut self, json: &str) -> Result<Response, ureq::Error> {
        self.call("/game_event", json)
    }

    #[allow(unused)]
    fn remove_game(&mut self, json: &str) -> Result<Response, ureq::Error> {
        // TODO consider removing
        self.call("/remove_game", json)
    }

    #[allow(unused)]
    fn game_heartbeat(&mut self, json: &str) -> Result<Response, ureq::Error> {
        self.call("/game_heartbeat", json)
    }
}

#[derive(Debug, Clone)]
struct SilentError;