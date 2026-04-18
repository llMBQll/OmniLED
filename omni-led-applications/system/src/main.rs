use all_smi::AllSmi;
use clap::Parser;
use omni_led_api::new_plugin;
use omni_led_derive::IntoProto;
use std::time;

mod cpu;
mod gpu;
mod mem;

#[tokio::main]
async fn main() {
    let options = Options::parse();
    let plugin = new_plugin!(&options.address);

    let smi = AllSmi::new().unwrap();
    loop {
        let data = SystemData {
            cpus: cpu::read_data(&smi),
            gpus: gpu::read_data(&smi),
            memory: mem::read_data(&smi),
        };
        plugin.update(data.into()).await.unwrap();

        tokio::time::sleep(time::Duration::from_secs(options.interval)).await;
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Options {
    #[clap(short, long)]
    address: String,

    /// Interval between getting new system data in seconds
    #[clap(short, long, default_value = "2")]
    interval: u64,
}

#[derive(IntoProto)]
#[proto(rename_all = PascalCase)]
struct SystemData {
    cpus: Vec<cpu::Data>,
    gpus: Vec<gpu::Data>,
    memory: Option<mem::Data>,
}
