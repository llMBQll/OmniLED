use std::collections::HashSet;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::time::Duration;

use mlua::{Lua, UserData};
use omni_led_derive::FromLuaValue;
use serde_json::Value;
use ureq::http::StatusCode;
use ureq::{Agent, Body};

use crate::common::lua_traits::LuaName;
use crate::common::user_data::{UserDataRef, set_unique_user_data};
use crate::renderer::buffer::{BitBuffer, BufferTrait};
use crate::script_handler::script_data_types::{DurationWrapper, Size};
use crate::settings::settings::Settings;

#[derive(Debug)]
pub enum Error {
    NotAvailable(String),
    Disconnected,
    BadRequest(ureq::Error),
    BadData(StatusCode, Body),
}

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Clone, Debug, FromLuaValue)]
#[mlua(impl_default)]
#[mlua(validate = Self::validate)]
pub struct ApiSettings {
    #[mlua(default = true)]
    enabled: bool,

    #[mlua(default = true)]
    register_heartbeat: bool,

    #[mlua(default = Duration::from_secs(15))]
    #[mlua(transform = DurationWrapper::transform)]
    deinitialize_timeout: Duration,
}

impl ApiSettings {
    fn validate(settings: &Self) -> mlua::Result<()> {
        if settings.deinitialize_timeout >= Duration::from_secs(1)
            && settings.deinitialize_timeout <= Duration::from_secs(60)
        {
            Ok(())
        } else {
            Err(mlua::Error::runtime(format!(
                "api_settings.deinitialize_timeout must be in range [1sec, 60sec] inclusive, got {:#?}",
                settings.deinitialize_timeout
            )))
        }
    }
}

pub struct Api {
    agent: Agent,
    address: Option<String>,
    counter: usize,
    sizes: HashSet<Size>,
    deinitialize_timeout: Duration,
}

const GAME: &str = "MBQ_OMNI_LED";
const GAME_DISPLAY_NAME: &str = "OmniLED";
const DEVELOPER: &str = "MBQ";

impl Api {
    pub fn load(lua: &Lua) {
        let settings = UserDataRef::<Settings>::load(lua);
        let api_settings = settings.get().steelseries_api.clone();

        if !api_settings.enabled {
            return;
        }

        // TODO register heartbeat event to fire on OMNILED.Update
        // send event only if last call was close to `timeout` ago

        let api = Self::new(api_settings.deinitialize_timeout);

        set_unique_user_data(lua, api);
    }

    fn new(deinitialize_timeout: Duration) -> Self {
        Self {
            agent: Agent::new_with_defaults(),
            address: None,
            counter: 0,
            sizes: HashSet::new(),
            deinitialize_timeout,
        }
    }

    pub fn register_size(&mut self, size: Size) {
        self.sizes.insert(size); // TODO see if it makes sense to track if size handler was bound
    }

    pub fn update(&mut self, size: &Size, data: &[u8]) -> Result<()> {
        let update = serde_json::json!({
            "game": GAME,
            "event": Self::event_name_for_size(&size),
            "data": {
                "value": self.counter,
                "frame": {
                    Self::data_field_for_size(&size): data
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
            "deinitialize_timer_length_ms": self.deinitialize_timeout.as_millis()
        });
        self.game_metadata(serde_json::to_string(&metadata).unwrap().as_str())?;

        let sizes = self.sizes.clone();
        for size in sizes {
            // Use buffer type for correctly handling widths not divisible by 8.
            // In practice no currently available device requires it though.
            let buffer = BitBuffer::new(Size {
                width: size.width,
                height: size.height,
            });
            let empty_data = buffer.bytes();

            let handler = serde_json::json!({
                "game": GAME,
                "event": Self::event_name_for_size(&size),
                "handlers": [{
                    "datas": [{
                        "has-text": false,
                        "image-data": empty_data,
                    }],
                    "device-type": Self::device_type_for_size(&size),
                    "mode": "screen",
                    "zone": "one",
                }]
            });
            self.bind_game_event(serde_json::to_string(&handler).unwrap().as_str())?;
        }

        // todo!("Register heartbeat event")

        Ok(())
    }

    fn unregister(&mut self) -> Result<()> {
        let remove_game = serde_json::json!({ "game": GAME });

        self.remove_game(serde_json::to_string(&remove_game).unwrap().as_str())
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
            .post(url.as_str())
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

    fn read_address() -> Result<String> {
        let program_data =
            std::env::var("PROGRAMDATA").expect("PROGRAMDATA env variable not found");
        let dir = format!("{}/SteelSeries/SteelSeries Engine 3", program_data);
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

    fn event_name_for_size(size: &Size) -> String {
        format!("UPDATE-{}X{}", size.width, size.height)
    }

    fn data_field_for_size(size: &Size) -> String {
        format!("image-data-{}x{}", size.width, size.height)
    }

    fn device_type_for_size(size: &Size) -> String {
        format!("screened-{}x{}", size.width, size.height)
    }
}

impl Drop for Api {
    fn drop(&mut self) {
        _ = self.unregister();
    }
}

impl LuaName for Api {
    const NAME: &str = "SteelSeriesApi";
}

impl UserData for Api {}
