use api::Api;
use chrono::prelude::*;
use serde::Serialize;
use std::{env, thread, time};

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
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

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct Time {
    hours: u32,
    minutes: u32,
    seconds: u32,
    month_day: u32,
    week_day: u32,
    month: u32,
    year: i32,
}

const NAME: &str = "CLOCK";

fn main() {
    let args: Vec<String> = env::args().collect();
    let address = args[1].as_str();
    let api = Api::new(address, NAME);

    // Send initial data that will not be updated
    api.update(&Names::new());

    let mut time = Time {
        hours: 0,
        minutes: 0,
        seconds: 0,
        month_day: 0,
        week_day: 0,
        month: 0,
        year: 0,
    };

    loop {
        let local = Local::now();
        if local.second() == time.seconds {
            thread::sleep(time::Duration::from_millis(10));
            continue;
        }
        time.seconds = local.second();
        time.minutes = local.minute();
        time.hours = local.hour();
        time.month_day = local.day();
        time.week_day = local.weekday().num_days_from_monday();
        time.month = local.month0();
        time.year = local.year();
        api.update(&time);
        thread::sleep(time::Duration::from_millis(500));
    }
}
