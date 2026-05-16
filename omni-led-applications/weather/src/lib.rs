use std::time::Duration;

use clap::Parser;
use log::debug;
use omni_led_api::cli_types::{TEMPERATURE_UNIT_DEFAULT, TEMPERATURE_UNIT_OPTIONS};
use omni_led_api::rust_api::OmniLedApi;
use omni_led_api::types::Table;
use omni_led_api::{new_plugin, plugin::Plugin};
use omni_led_derive::{IntoProto, plugin_entry};
use ureq::Agent;

mod weather_api;

#[plugin_entry]
pub fn omni_led_run(api: OmniLedApi, args: Vec<&str>) {
    let plugin = new_plugin!(api);
    let options = Options::parse_from(args);

    debug!("{:?}", options);

    load_and_send_images(&plugin);

    let (coordinates, name) = match &options.selector {
        Selector::In(name) => (get_coordinates_from_name(name), &name.city),
        Selector::At(coordinates) => (coordinates.clone(), &"N/A".to_string()),
    };

    debug!("Mapped to {} at {:?}", name, coordinates);

    loop {
        let weather = weather_api::get_weather(&coordinates, name, &options);
        plugin.update(weather.into()).unwrap();

        std::thread::sleep(options.interval);
    }
}

fn load_and_send_images(plugin: &Plugin) {
    let images = weather_api::load_images();

    let mut table = Table::default();
    for (name, image) in images {
        table.items.insert(name.into(), image.into());
    }
    plugin.update(table).unwrap();
}

fn get_coordinates_from_name(name: &Name) -> Coordinates {
    const GEOCODING_URL_BASE: &str = "https://geocoding-api.open-meteo.com/v1/search";

    let agent = Agent::new_with_defaults();
    let mut res = agent
        .get(GEOCODING_URL_BASE)
        .query("name", &name.city)
        .call()
        .unwrap();
    let results: Results = res.body_mut().read_json().unwrap();

    let mut mapped = results
        .results
        .iter()
        .enumerate()
        .filter_map(|(index, data)| {
            let admin_matches = name.administrative.is_none()
                || name.administrative == data.admin1
                || name.administrative == data.admin2
                || name.administrative == data.admin3
                || name.administrative == data.admin4;

            let code_matches = match &name.country_code {
                Some(country_code) => *country_code == data.country_code,
                None => true,
            };

            match admin_matches && code_matches {
                true => Some((
                    index,
                    Coordinates {
                        latitude: data.latitude,
                        longitude: data.longitude,
                    },
                )),
                false => None,
            }
        });

    match mapped.next() {
        Some((index, coordinates)) => {
            debug!("Found '{}': {:?}", name.city, results.results[index]);
            coordinates
        }
        None => {
            if results.results.is_empty() {
                panic!("Didn't find city '{}'", name.city);
            } else {
                let mut found_entries = String::new();
                for (index, entry) in results.results.iter().enumerate() {
                    found_entries +=
                        &format!("\n  [{}/{}] {:?}", index + 1, results.results.len(), entry);
                }
                panic!(
                    "Didn't match any entries for {:?}. Entries:{}",
                    name, found_entries
                );
            }
        }
    }
}

#[derive(IntoProto)]
#[proto(rename_all = PascalCase)]
struct WeatherData {
    city: String,
    image_key: &'static str,
    is_day: bool,
    latitude: f64,
    longitude: f64,
    temperature: f64,
    temperature_unit: String,
    weather_description: String,
    wind_direction: u32,
    wind_speed: f64,
    wind_speed_unit: String,
    update_hour: u32,
    update_minute: u32,
}

#[derive(clap::Args, Debug, Clone)]
struct Coordinates {
    latitude: f64,
    longitude: f64,
}

#[derive(clap::Args, Debug)]
struct Name {
    city: String,

    #[clap(long)]
    country_code: Option<String>,

    #[clap(long)]
    administrative: Option<String>,
}

#[derive(clap::Subcommand, Debug)]
enum Selector {
    /// Selects location by city name
    In(Name),

    /// Selects location by coordinates
    At(Coordinates),
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Options {
    #[clap(subcommand)]
    selector: Selector,

    /// Interval between getting new weather data
    #[clap(short, long, value_parser = humantime::parse_duration, default_value = "15min")]
    interval: Duration,

    /// Temperature unit
    #[clap(short, long, value_parser = TEMPERATURE_UNIT_OPTIONS, default_value = TEMPERATURE_UNIT_DEFAULT)]
    temperature_unit: String,

    /// Wind speed unit
    #[clap(short, long, value_parser = ["km/h", "m/s", "mph", "knots"], default_value = "km/h")]
    wind_speed_unit: String,
}

#[derive(serde::Deserialize, Debug)]
struct GeocodingData {
    pub latitude: f64,
    pub longitude: f64,
    pub country_code: String,
    pub admin1: Option<String>,
    pub admin2: Option<String>,
    pub admin3: Option<String>,
    pub admin4: Option<String>,
}

#[derive(serde::Deserialize, Debug)]
struct Results {
    pub results: Vec<GeocodingData>,
}
