use all_smi::AllSmi;
use omni_led_api::cli_types::TemperatureUnit;
use omni_led_derive::IntoProto;

use crate::util::convert;

#[derive(IntoProto)]
#[proto(rename_all = PascalCase)]
pub struct Data {
    name: String,
    utilization: f64,
    temperature: f64,
    power_consumption: f64,
    frequency: u32,
    used_memory: u64,
    total_memory: u64,
}

pub fn read_data(smi: &AllSmi, temperature_unit: TemperatureUnit) -> Vec<Data> {
    smi.get_gpu_info()
        .into_iter()
        .map(|data| Data {
            name: data.name,
            utilization: data.utilization,
            temperature: convert(data.temperature, temperature_unit),
            power_consumption: data.power_consumption,
            frequency: data.frequency,
            used_memory: data.used_memory,
            total_memory: data.total_memory,
        })
        .collect()
}
