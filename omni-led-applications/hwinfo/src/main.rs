use clap::Parser;
use log::warn;
use omni_led_api::plugin::Plugin;
use omni_led_derive::IntoProto;
use std::time::Duration;
use sysinfo::{MINIMUM_CPU_UPDATE_INTERVAL, System};

const NAME: &str = "HWINFO";

#[tokio::main]
async fn main() {
    let options = Options::parse();
    let mut plugin = Plugin::new(NAME, &options.address).await.unwrap();
    let mut sys = System::new_all();

    let mut interval = options.interval;
    if interval < MINIMUM_CPU_UPDATE_INTERVAL {
        warn!(
            "Setting update interval to a safe minimum value {}ms (got {}ms as input)",
            MINIMUM_CPU_UPDATE_INTERVAL.as_nanos() as f64 / 1_000_000_000.0,
            options.interval.as_nanos() as f64 / 1_000_000_000.0
        );
        interval = MINIMUM_CPU_UPDATE_INTERVAL;
    }

    let info = SystemInfo {
        kernel_version: System::kernel_long_version(),
        os_version: System::long_os_version().unwrap_or(String::from("unknown")),
        host_name: System::host_name().unwrap_or(String::from("unknown")),
        cpu_arch: System::cpu_arch(),
    };
    plugin.update(info.into()).await.unwrap();

    sys.refresh_cpu_all();
    let mut runtime_info = RuntimeInfo::default();
    for cpu in sys.cpus() {
        runtime_info.cpus.push(CpuInfo {
            name: cpu.name().to_string(),
            brand: cpu.brand().to_string(),
            frequency: 0,
            usage: 0.0,
        })
    }
    tokio::time::sleep(MINIMUM_CPU_UPDATE_INTERVAL).await;

    loop {
        sys.refresh_cpu_all();
        sys.refresh_memory();

        assert_eq!(sys.cpus().len(), runtime_info.cpus.len());
        runtime_info.total_cpu_usage = sys.global_cpu_usage();
        for (i, cpu) in sys.cpus().iter().enumerate() {
            runtime_info.cpus[i].frequency = cpu.frequency();
            runtime_info.cpus[i].usage = cpu.cpu_usage();
        }

        runtime_info.total_memory = sys.total_memory();
        runtime_info.used_memory = sys.used_memory();
        runtime_info.total_swap = sys.total_swap();
        runtime_info.used_swap = sys.used_swap();

        plugin.update(runtime_info.clone().into()).await.unwrap();

        tokio::time::sleep(interval).await;
    }
}

#[derive(IntoProto, Default)]
#[proto(rename_all = PascalCase)]
struct SystemInfo {
    kernel_version: String,
    os_version: String,
    host_name: String,
    cpu_arch: String,
}

#[derive(IntoProto, Default, Clone)]
#[proto(rename_all = PascalCase)]
struct RuntimeInfo {
    total_memory: u64,
    used_memory: u64,
    total_swap: u64,
    used_swap: u64,
    total_cpu_usage: f32,
    cpus: Vec<CpuInfo>,
}

#[derive(IntoProto, Clone)]
#[proto(rename_all = PascalCase)]
struct CpuInfo {
    name: String,
    brand: String,
    frequency: u64,
    usage: f32,
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Options {
    #[clap(short, long)]
    address: String,

    #[clap(short, long, value_parser = parse_duration)]
    interval: Duration,
}

fn parse_duration(arg: &str) -> Result<Duration, std::num::ParseIntError> {
    Ok(Duration::from_millis(arg.parse()?))
}
