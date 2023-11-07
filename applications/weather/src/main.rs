use api::Api;
use chrono::Timelike;
use clap::Parser;
use std::{collections::HashMap, thread, time};
use ureq::Agent;

const NAME: &str = "WEATHER";

fn main() {
    let options = Options::parse();

    let api = Api::new(&options.address, NAME);

    let (coordinates, name) = match options.selector {
        Selector::In(name) => (get_coordinates_from_name(&name), name.city),
        Selector::At(coordinates) => (coordinates, "N/A".to_string()),
    };

    let agent = Agent::new();

    loop {
        let weather = get_weather(&agent, &coordinates, &name);
        api.update(&weather);

        thread::sleep(time::Duration::from_secs(15 * 60));
    }
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

    let mut results = results.into_iter().filter_map(|data| {
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

fn get_weather(agent: &Agent, coordinates: &Coordinates, city: &String) -> Data {
    const OPEN_METEO_BASE: &str = "https://api.open-meteo.com/v1/forecast";

    let res = agent
        .get(OPEN_METEO_BASE)
        .query("current_weather", "true")
        .query("latitude", &coordinates.latitude.to_string())
        .query("longitude", &coordinates.longitude.to_string())
        .call()
        .unwrap();
    let result: WeatherResult = res.into_json().unwrap();

    let time = format!("{}:00-00:00", result.current_weather.time);
    let time: chrono::DateTime<chrono::Local> = chrono::DateTime::parse_from_rfc3339(time.as_str())
        .unwrap()
        .into();

    Data {
        latitude: result.latitude,
        longitude: result.longitude,
        temperature: result.current_weather.temperature,
        wind_speed: result.current_weather.windspeed,
        wind_direction: result.current_weather.winddirection,
        weather_code: result.current_weather.weathercode,
        is_day: result.current_weather.is_day != 0,
        update_hour: time.hour(),
        update_minute: time.minute(),
        city: city.clone(),
    }
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct Data {
    latitude: f64,
    longitude: f64,
    temperature: f64,
    wind_speed: f64,
    wind_direction: u32,
    weather_code: u32,
    is_day: bool,
    update_hour: u32,
    update_minute: u32,
    city: String,
}

#[derive(serde::Deserialize)]
struct CurrentWeatherResult {
    temperature: f64,
    windspeed: f64,
    winddirection: u32,
    weathercode: u32,
    is_day: u32,
    time: String,
}

#[derive(serde::Deserialize)]
struct WeatherResult {
    latitude: f64,
    longitude: f64,
    current_weather: CurrentWeatherResult,

    #[serde(flatten)]
    pub _other: HashMap<String, serde_json::Value>,
}

#[derive(clap::Args)]
struct Coordinates {
    latitude: f64,
    longitude: f64,
}

#[derive(clap::Args)]
struct Name {
    city: String,

    #[clap(long)]
    country_code: Option<String>,

    #[clap(long)]
    administrative: Option<String>,
}

#[derive(clap::Subcommand)]
enum Selector {
    /// Selects location by city name
    In(Name),

    /// Selects location by coordinates
    At(Coordinates),
}

#[derive(Parser)]
#[command(author, version, about)]
struct Options {
    #[clap(subcommand)]
    selector: Selector,

    /// Server address to which weather information will be sent
    #[clap(short, long)]
    address: String,

    /// Interval between getting new weather data in minutes
    #[clap(short, long, default_value = "15")]
    interval: u32,

    /// Temperature unit
    #[clap(short, long, value_parser = ["C", "Celsius", "F", "Fahrenheit"], default_value = "Celsius", ignore_case = true)]
    unit: String,
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

impl IntoIterator for Results {
    type Item = GeocodingData;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.results.into_iter()
    }
}
