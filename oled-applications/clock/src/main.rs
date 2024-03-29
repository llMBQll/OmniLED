use chrono::prelude::*;
use oled_api::types::{Array, Field, Table};
use oled_api::Api;
use std::{env, thread, time};

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

impl Into<Table> for Names {
    fn into(self) -> Table {
        let transform_vec = |vec: Vec<&str>| -> Field {
            let array = Array {
                items: vec.iter().map(|entry| (*entry).into()).collect(),
            };

            array.into()
        };

        let mut table = Table::default();
        table
            .items
            .insert("DayNames".to_owned(), transform_vec(self.day_names));
        table
            .items
            .insert("MonthNames".to_owned(), transform_vec(self.month_names));
        table
    }
}

struct Time {
    hours: u32,
    minutes: u32,
    seconds: u32,
    month_day: u32,
    week_day: u32,
    month: u32,
    year: i32,
}

impl Into<Table> for &Time {
    fn into(self) -> Table {
        let mut table = Table::default();
        table.items.insert("Hours".to_owned(), self.hours.into());
        table
            .items
            .insert("Minutes".to_owned(), self.minutes.into());
        table
            .items
            .insert("Seconds".to_owned(), self.seconds.into());
        table
            .items
            .insert("MonthDay".to_owned(), self.month_day.into());
        table
            .items
            .insert("WeekDay".to_owned(), self.week_day.into());
        table.items.insert("Month".to_owned(), self.month.into());
        table.items.insert("Year".to_owned(), self.year.into());
        table
    }
}

const NAME: &str = "CLOCK";

fn main() {
    let args: Vec<String> = env::args().collect();
    let address = args[1].as_str();
    let api = Api::new(address, NAME);

    // Send initial data that will not be updated
    api.update(Names::new().into());

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
        api.update((&time).into());
        thread::sleep(time::Duration::from_millis(500));
    }
}
