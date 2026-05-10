use omni_led_api::cli_types::TemperatureUnit;

pub fn convert(temperature: u32, output_unit: TemperatureUnit) -> f64 {
    match output_unit {
        TemperatureUnit::Celsius => temperature as f64,
        TemperatureUnit::Fahrenheit => temperature as f64 * 1.8 + 32.0,
    }
}
