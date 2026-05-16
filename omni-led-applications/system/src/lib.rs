use all_smi::AllSmi;
use clap::Parser;
use log::debug;
use omni_led_api::{
    cli_types::{TEMPERATURE_UNIT_DEFAULT, TEMPERATURE_UNIT_OPTIONS, TemperatureUnit},
    new_plugin,
};
use omni_led_derive::IntoProto;
use std::time::{Duration, Instant};

mod cpu;
mod gpu;
mod mem;
mod util;

// TODO wrap entry poing into a macro
#[unsafe(no_mangle)]
pub extern "C" fn omni_led_run(
    api: omni_led_api::c_api::OmniLedApi,
    argc: ::std::os::raw::c_int,
    argv: *mut *mut ::std::os::raw::c_char,
) {
    let plugin = new_plugin!(api);

    let args = omni_led_api::rust_api::argv_to_slice(argc, argv);
    debug!("{:?}", args);

    let options = Options::parse_from(args);

    let temperature_unit: TemperatureUnit = options.temperature_unit.into();

    let smi = AllSmi::new().unwrap();
    loop {
        let begin = Instant::now();

        let data = SystemData {
            cpus: cpu::read_data(&smi, temperature_unit),
            gpus: gpu::read_data(&smi, temperature_unit),
            memory: mem::read_data(&smi),
            temperature_unit: temperature_unit.unit(),
        };
        plugin.update(data.into()).unwrap();

        std::thread::sleep(options.interval.saturating_sub(begin.elapsed()));
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Options {
    /// Interval between getting new system data
    #[clap(short, long, value_parser = humantime::parse_duration, default_value = "2sec")]
    interval: Duration,

    /// Temperature unit
    #[clap(short, long, value_parser = TEMPERATURE_UNIT_OPTIONS, default_value = TEMPERATURE_UNIT_DEFAULT)]
    temperature_unit: String,
}

#[derive(IntoProto)]
#[proto(rename_all = PascalCase)]
struct SystemData {
    cpus: Vec<cpu::Data>,
    gpus: Vec<gpu::Data>,
    memory: Option<mem::Data>,
    temperature_unit: char,
}
