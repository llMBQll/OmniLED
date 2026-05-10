#[derive(Clone, Copy)]
pub enum TemperatureUnit {
    Celsius,
    Fahrenheit,
}

impl TemperatureUnit {
    pub const fn name(self) -> &'static str {
        match self {
            TemperatureUnit::Celsius => "Celsius",
            TemperatureUnit::Fahrenheit => "Fahrenheit",
        }
    }

    pub const fn unit(self) -> char {
        match self {
            TemperatureUnit::Celsius => 'C',
            TemperatureUnit::Fahrenheit => 'F',
        }
    }
}

impl From<&str> for TemperatureUnit {
    fn from(value: &str) -> Self {
        match value {
            "C" | "Celsius" => Self::Celsius,
            "F" | "Fahrenheit" => Self::Fahrenheit,
            _ => unreachable!("Should have been validated by clap by this time"),
        }
    }
}

impl From<String> for TemperatureUnit {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

pub const TEMPERATURE_UNIT_OPTIONS: [&str; 4] = ["C", "Celsius", "F", "Fahrenheit"];
pub const TEMPERATURE_UNIT_DEFAULT: &str = TemperatureUnit::Celsius.name();
