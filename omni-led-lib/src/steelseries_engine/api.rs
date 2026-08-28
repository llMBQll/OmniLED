use std::collections::HashSet;
use std::fs::File;
use std::io::BufReader;
use std::time::{Duration, Instant};

use log::error;
use mlua::{Lua, UserData};
use omni_led_derive::FromLuaValue;
use serde_json::Value;
use ureq::Agent;
use ureq::http::StatusCode;

use crate::common::lua_traits::LuaName;
use crate::common::user_data::{UserDataRef, set_unique_user_data};
use crate::events::events::Events;
use crate::renderer::buffer::{BitBuffer, BufferTrait};
use crate::script_handler::script_data_types::{DurationWrapper, EventKey, Size};
use crate::settings::settings::Settings;

#[derive(Clone, Debug, FromLuaValue)]
#[mlua(impl_default)]
#[mlua(validate = Self::validate)]
pub struct ApiSettings {
    #[mlua(default = Self::default_config_path())]
    config_path: Option<String>,

    #[mlua(default = cfg!(not(target_os = "linux")))]
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

    #[cfg(target_os = "linux")]
    fn default_config_path() -> Option<String> {
        None
    }

    #[cfg(target_os = "macos")]
    fn default_config_path() -> Option<String> {
        Some(String::from(
            "/Library/Application Support/SteelSeries Engine 3/coreProps.json",
        ))
    }

    #[cfg(target_os = "windows")]
    fn default_config_path() -> Option<String> {
        match std::env::var("PROGRAMDATA") {
            Ok(program_data) => Some(format!(
                "{}/SteelSeries/SteelSeries Engine 3/coreProps.json",
                program_data
            )),
            Err(err) => {
                error!("Failed to read PROGRAMDATA env variable: {}", err);
                None
            }
        }
    }
}

pub struct Api {
    agent: Agent,
    address: Option<String>,
    counter: usize,
    sizes: HashSet<Size>,
    deinitialize_timeout: Duration,
    last_update: Instant,
    config_path: String,
    error_count: usize,
}

const GAME: &str = "MBQ_OMNI_LED";
const GAME_DISPLAY_NAME: &str = "OmniLED";
const DEVELOPER: &str = "MBQ";

const BACKOFF_TABLE: &[Duration] = &[
    Duration::from_millis(500),
    Duration::from_millis(1000),
    Duration::from_millis(2000),
    Duration::from_millis(4000),
    Duration::from_millis(8000),
];

impl Api {
    pub fn load(lua: &Lua) {
        let settings = UserDataRef::<Settings>::load(lua);
        let api_settings = settings.get().steelseries_api.clone();

        if !api_settings.enabled {
            return;
        }

        let config_path = match api_settings.config_path {
            Some(config_path) => config_path,
            None => {
                error!("No config path found. SteelSeries Engine API will not be available");
                return;
            }
        };

        let api = Self::new(api_settings.deinitialize_timeout, config_path);
        set_unique_user_data(lua, api);

        if !api_settings.register_heartbeat {
            return;
        }

        let heartbeat_threshold = api_settings.deinitialize_timeout * 8 / 10;
        let heartbeat_payload = &serde_json::json!({
            "game": GAME,
        });
        let heartbeat_payload = serde_json::to_string(&heartbeat_payload).unwrap();
        let hearbeat = lua
            .create_function(move |lua: &Lua, _: ()| {
                let mut api = UserDataRef::<Self>::load(lua).get_mut();

                // Don't bother sending heartbeat if it's disconnected
                if api.address.is_some() && api.last_update.elapsed() > heartbeat_threshold {
                    if let Err(err) = api.heartbeat(&heartbeat_payload) {
                        error!("Failed to send heartbeat event: {:?}", err);
                    }
                }

                Ok(())
            })
            .unwrap();

        Events::register(
            EventKey::String(String::from("OMNILED.Update")),
            hearbeat,
            true,
        );
    }

    fn new(deinitialize_timeout: Duration, config_path: String) -> Self {
        Self {
            agent: Agent::new_with_defaults(),
            address: None,
            counter: 0,
            sizes: HashSet::new(),
            deinitialize_timeout,
            last_update: Instant::now(),
            config_path,
            error_count: 0,
        }
    }

    pub fn register_size(&mut self, size: Size) {
        self.sizes.insert(size); // TODO see if it makes sense to track if size handler was bound
    }

    pub fn update(&mut self, size: &Size, data: &[u8]) -> mlua::Result<()> {
        let update = serde_json::json!({
            "game": GAME,
            "event": Self::event_name_for_size(size),
            "data": {
                "value": self.counter,
                "frame": {
                    Self::data_field_for_size(size): data
                }
            }
        });
        self.counter += 1;

        self.game_event(serde_json::to_string(&update).unwrap().as_str())
    }

    fn register(&mut self) -> mlua::Result<()> {
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

        Ok(())
    }

    fn unregister(&mut self) -> mlua::Result<()> {
        let remove_game = serde_json::json!({ "game": GAME });

        self.remove_game(serde_json::to_string(&remove_game).unwrap().as_str())
    }

    fn game_metadata(&mut self, json: &str) -> mlua::Result<()> {
        self.call("/game_metadata", json)
    }

    fn bind_game_event(&mut self, json: &str) -> mlua::Result<()> {
        self.call("/bind_game_event", json)
    }

    fn game_event(&mut self, json: &str) -> mlua::Result<()> {
        self.call("/game_event", json)
    }

    fn remove_game(&mut self, json: &str) -> mlua::Result<()> {
        self.call("/remove_game", json)
    }

    fn heartbeat(&mut self, json: &str) -> mlua::Result<()> {
        self.call("/game_heartbeat", json)
    }

    fn try_reconnecting(&mut self) -> mlua::Result<()> {
        if self.address.is_some() {
            return Ok(());
        }

        self.address = Some(self.read_address()?);
        self.register()
    }

    fn call(&mut self, endpoint: &str, json: &str) -> mlua::Result<()> {
        if self.error_count != 0 {
            let index = std::cmp::min(self.error_count, BACKOFF_TABLE.len()) - 1;
            let elapsed = self.last_update.elapsed();
            if elapsed < BACKOFF_TABLE[index] {
                return Err(mlua::Error::runtime(format!(
                    "Skipping request, backoff remaining: {:?}",
                    BACKOFF_TABLE[index] - elapsed
                )));
            }
        }

        self.last_update = Instant::now();
        match self.call_impl(endpoint, json) {
            Ok(()) => {
                self.error_count = 0;
                Ok(())
            }
            Err(err) => {
                self.error_count += 1;
                Err(err)
            }
        }
    }

    fn call_impl(&mut self, endpoint: &str, json: &str) -> mlua::Result<()> {
        self.try_reconnecting()?;

        let address = self.address.as_ref().unwrap();

        let url = format!("http://{}{}", address, endpoint);
        let result = self
            .agent
            .post(url.as_str())
            .content_type("application/json")
            .send(json);

        match result {
            Ok(response) if response.status() == StatusCode::OK => Ok(()),
            Ok(response) => Err(mlua::Error::runtime(format!(
                "SteelSeries API request failed with status {}: {:?}",
                response.status(),
                response.body(),
            ))),
            Err(ureq::Error::HostNotFound) | Err(ureq::Error::Io(_)) => {
                self.address = None;
                Err(mlua::Error::runtime("SteelSeries API is disconnected"))
            }
            Err(error) => Err(mlua::Error::external(error)),
        }
    }

    fn read_address(&self) -> mlua::Result<String> {
        let file = File::open(&self.config_path).map_err(|error| {
            mlua::Error::runtime(format!("Couldn't open '{}': {}", self.config_path, error))
        })?;

        let reader = BufReader::new(file);
        let json: Value = serde_json::from_reader(reader).map_err(|error| {
            mlua::Error::runtime(format!("Couldn't parse properties json: {}", error))
        })?;

        json["address"].as_str().map(String::from).ok_or_else(|| {
            mlua::Error::runtime("Couldn't parse properties json: missing 'address' field")
        })
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
