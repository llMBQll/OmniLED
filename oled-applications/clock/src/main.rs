use chrono::prelude::*;
use clap::Parser;
use oled_api::Plugin;
use oled_derive::IntoProto;
use std::time;

#[derive(IntoProto)]
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

#[derive(Clone, IntoProto)]
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

#[tokio::main]
async fn main() {
    let options = Options::parse();
    let mut plugin = Plugin::new(NAME, &options.address).await.unwrap();

    // Send initial data that will not be updated
    plugin.update(Names::new().into()).await.unwrap();

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
            tokio::time::sleep(time::Duration::from_millis(10)).await;
            continue;
        }
        time.seconds = local.second();
        time.minutes = local.minute();
        time.hours = local.hour();
        time.month_day = local.day();
        time.week_day = local.weekday().number_from_monday();
        time.month = local.month();
        time.year = local.year();
        plugin.update(time.clone().into()).await.unwrap();
        tokio::time::sleep(time::Duration::from_millis(500)).await;
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Options {
    #[clap(short, long)]
    address: String,
}
