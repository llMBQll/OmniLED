use chrono::prelude::*;
use omni_led_api::{new_plugin, rust_api::OmniLedApi};
use omni_led_derive::{IntoProto, plugin_entry};
use std::time::{Duration, Instant};

#[derive(IntoProto)]
#[proto(rename_all = PascalCase)]
struct Names {
    day_names: Vec<&'static str>,
    month_names: Vec<&'static str>,
}

impl Names {
    pub fn new() -> Self {
        Self {
            day_names: vec![
                "Monday",
                "Tuesday",
                "Wednesday",
                "Thursday",
                "Friday",
                "Saturday",
                "Sunday",
            ],
            month_names: vec![
                "January",
                "February",
                "March",
                "April",
                "May",
                "June",
                "July",
                "August",
                "September",
                "October",
                "November",
                "December",
            ],
        }
    }
}

#[derive(IntoProto)]
#[proto(rename_all = PascalCase)]
struct Time {
    hours: u32,
    minutes: u32,
    seconds: u32,
    month_day: u32,
    week_day: u32,
    month: u32,
    year: i32,
}

#[plugin_entry]
pub fn omni_led_run(api: OmniLedApi, _args: Vec<&str>) {
    let plugin = new_plugin!(api);

    // Send initial data that will not be updated
    plugin.update(Names::new().into()).unwrap();

    let mut expected_update_time = Instant::now() + Duration::from_secs(1);
    loop {
        let local = Local::now();
        let time = Time {
            hours: local.hour(),
            minutes: local.minute(),
            seconds: local.second(),
            month_day: local.day(),
            week_day: local.weekday().number_from_monday(),
            month: local.month(),
            year: local.year(),
        };

        plugin.update(time.into()).unwrap();

        let sleep_duration = expected_update_time - Instant::now();
        std::thread::sleep(sleep_duration);
        expected_update_time += Duration::from_secs(1);
    }
}
