use lazy_static::lazy_static;
use log::error;
use serde_json::Value;
use std::collections::HashSet;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Mutex;
use ureq::{Agent, Error};

use crate::devices::device::Size;
use crate::renderer::buffer::{BitBuffer, BufferTrait};

pub fn init(size: Size) {
    API.lock().unwrap().register_size(size);
}

pub fn update(size: &Size, data: &[u8]) {
    API.lock().unwrap().update(size, data);
}

// TODO error (propagation) handling on raw api calls

lazy_static! {
    static ref API: Mutex<Api> = Mutex::new(Api::new());
}

struct Api {
    agent: Agent,
    address: Option<String>,
    counter: usize,
    sizes: HashSet<Size>,
}

const GAME: &str = "MBQ_STEELSERIES_OLED";
const GAME_DISPLAY_NAME: &str = "Steelseries OLED";
const DEVELOPER: &str = "MBQ";
const TIMEOUT: u32 = 60000;

impl Api {
    pub fn new() -> Self {
        Self {
            agent: Agent::new(),
            address: None,
            counter: 0,
            sizes: HashSet::new(),
        }
    }

    pub fn register_size(&mut self, size: Size) {
        self.sizes.insert(size);
    }

    pub fn update(&mut self, size: &Size, data: &[u8]) {
        let update = serde_json::json!({
            "game": GAME,
            "event": Self::get_event(&size),
            "data": {
                "value": self.counter,
                "frame": {
                    Self::get_image_data_field(&size): data
                }
            }
        });
        self.counter += 1;

        self.game_event(serde_json::to_string(&update).unwrap().as_str())
    }

    fn register(&mut self) {
        let metadata = serde_json::json!({
            "game": GAME,
            "game_display_name": GAME_DISPLAY_NAME,
            "developer": DEVELOPER,
            "deinitialize_timer_length_ms": TIMEOUT
        });
        self.game_metadata(serde_json::to_string(&metadata).unwrap().as_str());

        let sizes = self.sizes.clone();
        for size in sizes {
            // Use buffer type for correctly handling widths not divisible by 8 which in theory
            // should not happen as all currently available devices have 128 pixel width
            let buffer = BitBuffer::new(Size {
                width: size.width,
                height: size.height,
            });
            let empty_data = buffer.bytes();

            let handler = serde_json::json!({
                "game": GAME,
                "event": Self::get_event(&size),
                "handlers": [{
                    "datas": [{
                        "has-text": false,
                        "image-data": empty_data,
                    }],
                    "device-type": Self::get_device_type(&size),
                    "mode": "screen",
                    "zone": "one",
                }]
            });
            self.bind_game_event(serde_json::to_string(&handler).unwrap().as_str());
        }

        // todo!("Register heartbeat event")
    }

    fn unregister(&mut self) {
        let remove_game = serde_json::json!({ "game": GAME });

        self.remove_game(serde_json::to_string(&remove_game).unwrap().as_str());
    }

    fn game_metadata(&mut self, json: &str) {
        self.call("/game_metadata", json)
    }

    fn bind_game_event(&mut self, json: &str) {
        self.call("/bind_game_event", json)
    }

    fn game_event(&mut self, json: &str) {
        self.call("/game_event", json)
    }

    fn remove_game(&mut self, json: &str) {
        self.call("/remove_game", json)
    }

    // fn game_heartbeat(&mut self, json: &str) {
    //     self.call("/game_heartbeat", json)
    // }

    fn try_reconnecting(&mut self) {
        if self.address.is_none() {
            match Self::read_address() {
                Some(address) => {
                    self.address = Some(address);
                    self.register();
                }
                None => {}
            };
        }
    }

    fn call(&mut self, endpoint: &str, json: &str) {
        self.try_reconnecting();

        let address = match &self.address {
            Some(address) => address,
            None => return,
        };

        let url = format!("http://{}{}", address, endpoint);
        let result = self
            .agent
            .post(url.as_str())
            .set("Content-Type", "application/json")
            .send_string(json);
        match result {
            Ok(_) => {}
            Err(error) => match error {
                Error::Status(status, response) => {
                    error!(
                        "API call to {} failed with code {}: {:?}",
                        endpoint, status, response
                    );
                }
                Error::Transport(transport) => {
                    error!("API call to {} failed: {:?}", endpoint, transport);
                    self.address = None;
                }
            },
        }
    }

    fn read_address() -> Option<String> {
        // Missing directories are fatal errors
        let program_data =
            std::env::var("PROGRAMDATA").expect("PROGRAMDATA env variable not found");
        let dir = format!("{}/SteelSeries/SteelSeries Engine 3", program_data);
        if !Path::new(&dir).is_dir() {
            panic!("{} doesn't exist", dir);
        }

        // Rest of the errors are non-fatal, it is possible that Steelseries Engine is not yet ready
        let path = format!("{}/coreProps.json", dir);
        let file = match File::open(path) {
            Ok(file) => file,
            Err(_) => return None,
        };
        let reader = BufReader::new(file);
        let json: Value = match serde_json::from_reader(reader) {
            Ok(json) => json,
            Err(_) => return None,
        };

        json["address"]
            .as_str()
            .map(|address| String::from(address))
    }

    fn get_event(size: &Size) -> String {
        format!("UPDATE-{}X{}", size.width, size.height)
    }

    fn get_image_data_field(size: &Size) -> String {
        format!("image-data-{}x{}", size.width, size.height)
    }

    fn get_device_type(size: &Size) -> String {
        format!("screened-{}x{}", size.width, size.height)
    }
}

impl Drop for Api {
    fn drop(&mut self) {
        self.unregister()
    }
}
