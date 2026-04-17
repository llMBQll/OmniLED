use all_smi::AllSmi;
use omni_led_derive::IntoProto;

#[derive(IntoProto)]
#[proto(rename_all = PascalCase)]
pub struct Data {
    used: u64,
    total: u64,
    swap_used: u64,
    swap_total: u64,
    utilization: f64,
}

pub fn read_data(smi: &AllSmi) -> Option<Data> {
    smi.get_memory_info().into_iter().next().map(|data| Data {
        used: data.used_bytes,
        total: data.total_bytes,
        swap_used: data.swap_used_bytes,
        swap_total: data.swap_total_bytes,
        utilization: data.utilization,
    })
}
