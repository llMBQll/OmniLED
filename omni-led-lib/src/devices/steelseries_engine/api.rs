use lazy_static::lazy_static;
use std::collections::HashSet;
use std::sync::Mutex;
use ureq::http::StatusCode;
use ureq::{Agent, Body};

use crate::devices::device::Size;
use crate::renderer::buffer::{BitBuffer, BufferTrait};

pub fn register_size(size: Size) {
    API.lock().unwrap().register_size(size);
}

pub fn update(size: &Size, data: &[u8]) -> Result<()> {
    API.lock().unwrap().update(size, data)
}

#[derive(Debug)]
pub enum Error {
    NotAvailable(String),
    Disconnected,
    BadRequest(ureq::Error),
    BadData(StatusCode, Body),
}

pub type Result<T> = core::result::Result<T, Error>;

const GAME: &str = "MBQ_OMNI_LED";
const GAME_DISPLAY_NAME: &str = "OmniLED";
const DEVELOPER: &str = "MBQ";
const TIMEOUT: u32 = 60000;

lazy_static! {
    static ref API: Mutex<Api> = Mutex::new(Api::new());
}

struct Api {
    agent: Agent,
    address: Option<String>,
    counter: usize,
    sizes: HashSet<Size>,
}

impl Api {
    fn new() -> Self {
        Self {
            agent: Agent::new_with_defaults(),
            address: None,
            counter: 0,
            sizes: HashSet::new(),
        }
    }

    fn register_size(&mut self, size: Size) {
        self.sizes.insert(size);
    }

    fn update(&mut self, size: &Size, data: &[u8]) -> Result<()> {
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

    fn register(&mut self) -> Result<()> {
        let metadata = serde_json::json!({
            "game": GAME,
            "game_display_name": GAME_DISPLAY_NAME,
            "developer": DEVELOPER,
            "deinitialize_timer_length_ms": TIMEOUT
        });
        self.game_metadata(serde_json::to_string(&metadata).unwrap().as_str())?;

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
            self.bind_game_event(serde_json::to_string(&handler).unwrap().as_str())?;
        }

        // todo!("Register heartbeat event")

        Ok(())
    }

    fn unregister(&mut self) {
        let remove_game = serde_json::json!({ "game": GAME });

        _ = self.remove_game(serde_json::to_string(&remove_game).unwrap().as_str());
    }

    fn game_metadata(&mut self, json: &str) -> Result<()> {
        self.call("/game_metadata", json)
    }

    fn bind_game_event(&mut self, json: &str) -> Result<()> {
        self.call("/bind_game_event", json)
    }

    fn game_event(&mut self, json: &str) -> Result<()> {
        self.call("/game_event", json)
    }

    fn remove_game(&mut self, json: &str) -> Result<()> {
        self.call("/remove_game", json)
    }

    // fn game_heartbeat(&mut self, json: &str) {
    //     self.call("/game_heartbeat", json)
    // }

    fn try_reconnecting(&mut self) -> Result<()> {
        match self.address {
            Some(_) => Ok(()),
            None => match Self::read_address() {
                Ok(address) => {
                    self.address = Some(address);
                    self.register()
                }
                Err(error) => Err(error),
            },
        }
    }

    fn call(&mut self, endpoint: &str, json: &str) -> Result<()> {
        self.try_reconnecting()?;

        let address = match &self.address {
            Some(address) => address,
            None => return Err(Error::Disconnected),
        };

        let url = format!("http://{}{}", address, endpoint);
        let result = self
            .agent
            .post(&url)
            .content_type("application/json")
            .send(json);

        match result {
            Ok(response) => {
                let status = response.status();
                if status == StatusCode::OK {
                    Ok(())
                } else {
                    Err(Error::BadData(status, response.into_body()))
                }
            }
            Err(error) => match error {
                ureq::Error::HostNotFound => Err(Error::Disconnected),
                other => Err(Error::BadRequest(other)),
            },
        }
    }

    #[cfg(target_os = "linux")]
    fn read_address() -> Result<String> {
        Err(Error::NotAvailable(
            "SteelSeries Engine does not work on Linux".to_string(),
        ))
    }

    #[cfg(not(target_os = "linux"))]
    fn read_address() -> Result<String> {
        use serde_json::Value;
        use std::fs::File;
        use std::io::BufReader;
        use std::path::Path;

        #[cfg(target_os = "windows")]
        let dir = {
            let program_data =
                std::env::var("PROGRAMDATA").expect("PROGRAMDATA env variable not found");
            format!("{}/SteelSeries/SteelSeries Engine 3", program_data)
        };

        #[cfg(target_os = "macos")]
        let dir = String::from("/Library/Application Support/SteelSeries Engine 3");

        if !Path::new(&dir).is_dir() {
            return Err(Error::NotAvailable(format!(
                "SteelSeries Engine directory '{}' doesn't exist",
                dir
            )));
        }

        let path = format!("{}/coreProps.json", dir);
        let file = match File::open(&path) {
            Ok(file) => file,
            Err(error) => {
                return Err(Error::NotAvailable(format!(
                    "Couldn't open '{}'. {}",
                    path, error
                )));
            }
        };

        let reader = BufReader::new(file);
        let json: Value = match serde_json::from_reader(reader) {
            Ok(json) => json,
            Err(error) => {
                return Err(Error::NotAvailable(format!(
                    "Couldn't parse properties json. {}",
                    error
                )));
            }
        };

        json["address"]
            .as_str()
            .map(|address| String::from(address))
            .ok_or(Error::NotAvailable(
                "Couldn't parse properties json. Didn't find 'address' field".to_string(),
            ))
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
