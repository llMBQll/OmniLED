use api::Api;
use chrono::prelude::*;
use serde::Serialize;
use std::{env, thread, time};

#[allow(non_snake_case)]
#[derive(Serialize)]
struct Names {
    DayNames: Vec<&'static str>,
    MonthNames: Vec<&'static str>,
}

impl Names {
    pub fn new() -> Self {
        Self {
            DayNames: vec![
                "Monday",
                "Tuesday",
                "Wednesday",
                "Thursday",
                "Friday",
                "Saturday",
                "Sunday",
            ],
            MonthNames: vec![
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

#[allow(non_snake_case)]
#[derive(Serialize)]
struct Time {
    Hours: u32,
    Minutes: u32,
    Seconds: u32,
    MonthDay: u32,
    WeekDay: u32,
    Month: u32,
    Year: i32,
}

const NAME: &str = "CLOCK";

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let address = args[1].as_str();
    let api = Api::new(String::from(address), String::from(NAME));

    // Send initial data that will not be updated
    api.update(&Names::new());

    let mut time = Time {
        Hours: 0,
        Minutes: 0,
        Seconds: 0,
        MonthDay: 0,
        WeekDay: 0,
        Month: 0,
        Year: 0,
    };

    loop {
        let local = Local::now();
        if local.second() == time.Seconds {
            thread::sleep(time::Duration::from_millis(10));
            continue;
        }
        time.Seconds = local.second();
        time.Minutes = local.minute();
        time.Hours = local.hour();
        time.MonthDay = local.day();
        time.WeekDay = local.weekday().num_days_from_monday();
        time.Month = local.month0();
        time.Year = local.year();
        api.update(&time);
        thread::sleep(time::Duration::from_millis(500));
    }
}
