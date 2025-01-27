/*
 * OmniLED is a software for displaying data on various OLED devices.
 * Copyright (C) 2024  Michał Bałabanow <m.balabanow@gmail.com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

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
