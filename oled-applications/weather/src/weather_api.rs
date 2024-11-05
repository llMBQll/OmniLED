use chrono::Timelike;
use oled_api::Image;
use serde::de;
use std::collections::HashMap;
use ureq::Agent;

use crate::{Coordinates, Options, WeatherData};

pub fn get_weather(
    agent: &Agent,
    coordinates: &Coordinates,
    city: &String,
    options: &Options,
) -> WeatherData {
    const OPEN_METEO_BASE: &str = "https://api.open-meteo.com/v1/forecast";

    let res = agent
        .get(OPEN_METEO_BASE)
        .query("current_weather", "true")
        .query("latitude", &coordinates.latitude.to_string())
        .query("longitude", &coordinates.longitude.to_string())
        .query(
            "wind_speed_unit",
            speed_unit_param(&options.wind_speed_unit),
        )
        .query(
            "temperature_unit",
            temperature_unit_param(&options.temperature_unit),
        )
        .call()
        .unwrap();
    let result: WeatherResult = res.into_json().unwrap();

    let time = format!("{}:00-00:00", result.current_weather.time);
    let time: chrono::DateTime<chrono::Local> = chrono::DateTime::parse_from_rfc3339(time.as_str())
        .unwrap()
        .into();

    let is_day = result.current_weather.is_day != 0;
    let weather = result.current_weather.weather_code;

    WeatherData {
        city: city.clone(),
        image_key: get_image_key(weather, is_day),
        is_day,
        latitude: result.latitude,
        longitude: result.longitude,
        temperature: result.current_weather.temperature,
        temperature_unit: temperature_unit_data(&options.temperature_unit).to_string(),
        update_hour: time.hour(),
        update_minute: time.minute(),
        weather_description: weather.to_desc(),
        wind_direction: result.current_weather.wind_direction,
        wind_speed: result.current_weather.wind_speed,
        wind_speed_unit: options.wind_speed_unit.clone(),
    }
}

fn speed_unit_param(unit: &str) -> &'static str {
    match unit {
        "km/h" => "kmh",
        "m/s" => "ms",
        "mph" => "mph",
        "knots" => "kn",
        _ => std::unreachable!(),
    }
}

fn temperature_unit_param(unit: &str) -> &'static str {
    match unit {
        "C" | "Celsius" => "celsius",
        "F" | "Fahrenheit" => "fahrenheit",
        _ => std::unreachable!(),
    }
}

fn temperature_unit_data(unit: &str) -> &'static str {
    match unit {
        "C" | "Celsius" => "C",
        "F" | "Fahrenheit" => "F",
        _ => std::unreachable!(),
    }
}

#[derive(serde::Deserialize)]
struct WeatherResult {
    latitude: f64,
    longitude: f64,
    current_weather: CurrentWeatherResult,

    #[serde(flatten)]
    pub _other: HashMap<String, serde_json::Value>,
}

#[derive(serde::Deserialize)]
struct CurrentWeatherResult {
    temperature: f64,
    #[serde(rename(deserialize = "windspeed"))]
    wind_speed: f64,
    #[serde(rename(deserialize = "winddirection"))]
    wind_direction: u32,
    #[serde(
        rename(deserialize = "weathercode"),
        deserialize_with = "map_from_weather_code"
    )]
    weather_code: Weather,
    is_day: u32,
    time: String,
}

#[derive(Debug, Copy, Clone)]
enum Weather {
    ClearSky,
    MainlyClear,
    PartlyCloudy,
    Overcast,
    Fog,
    DepositingRimeFog,
    LightDrizzle,
    ModerateDrizzle,
    DenseDrizzle,
    LightFreezingDrizzle,
    DenseFreezingDrizzle,
    SlightRain,
    ModerateRain,
    HeavyRain,
    SlightFreezingRain,
    HeavyFreezingRain,
    SlightSnowFall,
    ModerateSnowFall,
    HeavySnowFall,
    SnowGrains,
    SlightRainShowers,
    ModerateRainShowers,
    ViolentRainShowers,
    SlightSnowShowers,
    HeavySnowShowers,
    Thunderstorm,
    ThunderstormWithSlightHail,
    ThunderstormWithHeavyHail,
}

impl Weather {
    fn to_desc(&self) -> String {
        let string = format!("{:?}", self);
        let mut desc = String::new();
        let mut was_upper = true;
        for c in string.chars() {
            let is_upper = c.is_ascii_uppercase();
            if is_upper && was_upper {
                desc.push(' ');
            }
            desc.push(c);
            was_upper = is_upper;
        }
        desc
    }
}

fn map_from_weather_code<'de, D>(deserializer: D) -> Result<Weather, D::Error>
where
    D: de::Deserializer<'de>,
{
    // 0 	Clear sky
    // 1, 2, 3 	Mainly clear, partly cloudy, and overcast
    // 45, 48 	Fog and depositing rime fog
    // 51, 53, 55 	Drizzle: Light, moderate, and dense intensity
    // 56, 57 	Freezing Drizzle: Light and dense intensity
    // 61, 63, 65 	Rain: Slight, moderate and heavy intensity
    // 66, 67 	Freezing Rain: Light and heavy intensity
    // 71, 73, 75 	Snow fall: Slight, moderate, and heavy intensity
    // 77 	Snow grains
    // 80, 81, 82 	Rain showers: Slight, moderate, and violent
    // 85, 86 	Snow showers slight and heavy
    // 95 * 	Thunderstorm: Slight or moderate
    // 96, 99 * 	Thunderstorm with slight and heavy hail

    // const EXPECTED: &[u32] = &[
    //     0, 1, 2, 3, 45, 48, 51, 53, 55, 61, 63, 65, 66, 67, 71, 73, 75, 77, 80, 81, 82, 85, 86, 95,
    //     96, 99,
    // ];

    let code: u32 = de::Deserialize::deserialize(deserializer)?;
    let weather = match code {
        0 => Weather::ClearSky,
        1 => Weather::MainlyClear,
        2 => Weather::PartlyCloudy,
        3 => Weather::Overcast,
        45 => Weather::Fog,
        48 => Weather::DepositingRimeFog,
        51 => Weather::LightDrizzle,
        53 => Weather::ModerateDrizzle,
        55 => Weather::DenseDrizzle,
        56 => Weather::LightFreezingDrizzle,
        57 => Weather::DenseFreezingDrizzle,
        61 => Weather::SlightRain,
        63 => Weather::ModerateRain,
        65 => Weather::HeavyRain,
        66 => Weather::SlightFreezingRain,
        67 => Weather::HeavyFreezingRain,
        71 => Weather::SlightSnowFall,
        73 => Weather::ModerateSnowFall,
        75 => Weather::HeavySnowFall,
        77 => Weather::SnowGrains,
        80 => Weather::SlightRainShowers,
        81 => Weather::ModerateRainShowers,
        82 => Weather::ViolentRainShowers,
        85 => Weather::SlightSnowShowers,
        86 => Weather::HeavySnowShowers,
        95 => Weather::Thunderstorm,
        96 => Weather::ThunderstormWithSlightHail,
        99 => Weather::ThunderstormWithHeavyHail,
        _value => todo!("Handle error"),
    };

    Ok(weather)
}

fn get_image_key(weather: Weather, is_day: bool) -> &'static str {
    // TODO get more images and provide a better mapping

    match (is_day, weather) {
        (true, Weather::ClearSky) => "DAY_CLEAR",
        (false, Weather::ClearSky) => "NIGHT_CLEAR",

        (true, Weather::MainlyClear) | (true, Weather::PartlyCloudy) => "DAY_CLOUDS",
        (false, Weather::MainlyClear) | (false, Weather::PartlyCloudy) => "NIGHT_CLOUDS",

        (_, Weather::Overcast) => "CLOUDS",
        (_, Weather::Fog) | (_, Weather::DepositingRimeFog) => "FOG",

        (_, Weather::LightDrizzle)
        | (_, Weather::ModerateDrizzle)
        | (_, Weather::DenseDrizzle)
        | (_, Weather::LightFreezingDrizzle)
        | (_, Weather::DenseFreezingDrizzle)
        | (_, Weather::SlightRain)
        | (_, Weather::ModerateRain)
        | (_, Weather::HeavyRain)
        | (_, Weather::SlightFreezingRain)
        | (_, Weather::HeavyFreezingRain)
        | (_, Weather::SlightRainShowers)
        | (_, Weather::ModerateRainShowers)
        | (_, Weather::ViolentRainShowers) => "RAIN",

        (_, Weather::SlightSnowFall)
        | (_, Weather::ModerateSnowFall)
        | (_, Weather::HeavySnowFall)
        | (_, Weather::SnowGrains)
        | (_, Weather::SlightSnowShowers)
        | (_, Weather::HeavySnowShowers) => "SNOW",

        (_, Weather::Thunderstorm)
        | (_, Weather::ThunderstormWithSlightHail)
        | (_, Weather::ThunderstormWithHeavyHail) => "THUNDERSTORM",
    }
}

pub fn load_images() -> Vec<(&'static str, Image)> {
    const IMAGES: &[(&str, &[u8])] = &[
        ("DAY_CLEAR", include_bytes!("../assets/day_clear.png")),
        ("NIGHT_CLEAR", include_bytes!("../assets/night_clear.png")),
        ("DAY_CLOUDS", include_bytes!("../assets/day_clouds.png")),
        ("NIGHT_CLOUDS", include_bytes!("../assets/night_clouds.png")),
        ("CLOUDS", include_bytes!("../assets/clouds.png")),
        ("FOG", include_bytes!("../assets/fog.png")),
        ("RAIN", include_bytes!("../assets/rain.png")),
        ("SNOW", include_bytes!("../assets/snow.png")),
        ("THUNDERSTORM", include_bytes!("../assets/thunderstorm.png")),
    ];

    IMAGES
        .into_iter()
        .map(|(name, bytes)| {
            let mut image =
                image::load_from_memory_with_format(bytes, image::ImageFormat::Png).unwrap();
            image.invert();
            let grayscale = image.into_luma8();

            let image = Image {
                width: grayscale.width() as i64,
                height: grayscale.height() as i64,
                data: grayscale.into_raw(),
            };
            (*name, image)
        })
        .collect()
}
