use clap::Parser;
use log::debug;
use omni_led_api::plugin::Plugin;
use omni_led_api::types::Table;
use omni_led_derive::IntoProto;
use std::{collections::HashMap, time};
use ureq::Agent;

mod weather_api;

const NAME: &str = "WEATHER";

#[tokio::main]
async fn main() {
    let options = Options::parse();
    let mut plugin = Plugin::new(NAME, &options.address).await.unwrap();

    debug!("{:?}", options);

    load_and_send_images(&mut plugin).await;

    let (coordinates, name) = match &options.selector {
        Selector::In(name) => (get_coordinates_from_name(name), &name.city),
        Selector::At(coordinates) => (coordinates.clone(), &"N/A".to_string()),
    };

    debug!("Mapped to {} at {:?}", name, coordinates);

    let agent = Agent::new();

    loop {
        let weather = weather_api::get_weather(&agent, &coordinates, name, &options);
        plugin.update(weather.into()).await.unwrap();

        tokio::time::sleep(time::Duration::from_secs(options.interval * 60)).await;
    }
}

async fn load_and_send_images(plugin: &mut Plugin) {
    let images = weather_api::load_images();

    let mut table = Table::default();
    for (name, image) in images {
        table.items.insert(name.into(), image.into());
    }
    plugin.update(table).await.unwrap();
}

fn get_coordinates_from_name(name: &Name) -> Coordinates {
    const GEOCODING_URL_BASE: &str = "https://geocoding-api.open-meteo.com/v1/search";

    let agent = Agent::new();
    let res = agent
        .get(GEOCODING_URL_BASE)
        .query("name", &name.city)
        .call()
        .unwrap();
    let results: Results = res.into_json().unwrap();

    let mut results = results.results.into_iter().filter_map(|data| {
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
            true => Some(Coordinates {
                latitude: data.latitude,
                longitude: data.longitude,
            }),
            false => None,
        }
    });

    results
        .next()
        .expect("Couldn't find coordinates for the given query")
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

    /// Server address to which weather information will be sent
    #[clap(short, long)]
    address: String,

    /// Interval between getting new weather data in minutes
    #[clap(short, long, default_value = "15")]
    interval: u64,

    /// Temperature unit
    #[clap(short, long, value_parser = ["C", "Celsius", "F", "Fahrenheit"], default_value = "Celsius")]
    temperature_unit: String,

    /// Wind speed unit
    #[clap(short, long, value_parser = ["km/h", "m/s", "mph", "knots"], default_value = "km/h")]
    wind_speed_unit: String,
}

/// All GeocodingData fields, some (all?) of which are optional
/// Data not required for this application is stored in the HashMap 'other'
///     id          : i64
///     name        : String
///     latitude    : f64
///     longitude   : f64
///     elevation   : f64
///     timezone    : String
///     feature_code: String
///     country_code: String
///     country_id  : i64
///     population  : i64
///     postcodes   : Vec<String>
///     admin1      : String
///     admin2      : String
///     admin3      : String
///     admin4      : String
///     admin1_id   : i64
///     admin2_id   : i64
///     admin3_id   : i64
///     admin4_id   : i64
#[derive(serde::Deserialize, Debug)]
struct GeocodingData {
    pub latitude: f64,
    pub longitude: f64,
    pub country_code: String,
    pub admin1: Option<String>,
    pub admin2: Option<String>,
    pub admin3: Option<String>,
    pub admin4: Option<String>,

    #[serde(flatten)]
    pub _other: HashMap<String, serde_json::Value>,
}

#[derive(serde::Deserialize, Debug)]
struct Results {
    pub results: Vec<GeocodingData>,
}
