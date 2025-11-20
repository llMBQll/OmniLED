use chrono::prelude::*;
use clap::Parser;
use omni_led_api::plugin::Plugin;
use omni_led_derive::IntoProto;
use tokio::time::{Duration, Instant};

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

const NAME: &str = "CLOCK";

#[tokio::main]
async fn main() {
    let options = Options::parse();
    let mut plugin = Plugin::new(NAME, &options.address).await.unwrap();

    // Send initial data that will not be updated
    plugin.update(Names::new().into()).await.unwrap();

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

        plugin.update(time.into()).await.unwrap();

        tokio::time::sleep_until(expected_update_time).await;
        expected_update_time += Duration::from_secs(1);
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Options {
    #[clap(short, long)]
    address: String,
}
