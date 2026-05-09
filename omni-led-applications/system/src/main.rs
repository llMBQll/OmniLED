use all_smi::AllSmi;
use clap::Parser;
use omni_led_api::new_plugin;
use omni_led_derive::IntoProto;
use std::time::{Duration, Instant};

mod cpu;
mod gpu;
mod mem;

#[tokio::main]
async fn main() {
    let options = Options::parse();
    let plugin = new_plugin!(&options.address);

    let smi = AllSmi::new().unwrap();
    loop {
        let begin = Instant::now();

        let data = SystemData {
            cpus: cpu::read_data(&smi),
            gpus: gpu::read_data(&smi),
            memory: mem::read_data(&smi),
        };
        plugin.update(data.into()).await.unwrap();

        tokio::time::sleep(options.interval.saturating_sub(begin.elapsed())).await;
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Options {
    #[clap(short, long)]
    address: String,

    /// Interval between getting new system data
    #[clap(short, long, value_parser = humantime::parse_duration, default_value = "2sec")]
    interval: Duration,
}

#[derive(IntoProto)]
#[proto(rename_all = PascalCase)]
struct SystemData {
    cpus: Vec<cpu::Data>,
    gpus: Vec<gpu::Data>,
    memory: Option<mem::Data>,
}
