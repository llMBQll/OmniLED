use all_smi::{AllSmi, device::CoreType};
use omni_led_derive::IntoProto;

#[derive(IntoProto)]
#[proto(rename_all = PascalCase)]
pub struct Data {
    name: String,
    utilization: f64,
    temperature: Option<u32>,
    power_consumption: Option<f64>,
    cores: Vec<CoreData>,
}

#[derive(IntoProto)]
#[proto(rename_all = PascalCase)]
pub struct CoreData {
    id: u32,
    utilization: f64,
    r#type: &'static str,
}

pub fn read_data(smi: &AllSmi) -> Vec<Data> {
    smi.get_cpu_info()
        .into_iter()
        .map(|data| Data {
            name: data.cpu_model,
            utilization: data.utilization,
            temperature: data.temperature,
            power_consumption: data.power_consumption,
            cores: data
                .per_core_utilization
                .into_iter()
                .map(|data| CoreData {
                    id: data.core_id,
                    utilization: data.utilization,
                    r#type: map_core_type(data.core_type),
                })
                .collect(),
        })
        .collect()
}

fn map_core_type(r#type: CoreType) -> &'static str {
    match r#type {
        CoreType::Super => "Super",
        CoreType::Performance => "Performance",
        CoreType::Efficiency => "Efficiency",
        CoreType::Standard => "Standard",
    }
}
