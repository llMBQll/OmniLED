use omni_led_derive::IntoProto;
use std::time::Duration;

#[derive(Clone, IntoProto, Default, Debug)]
#[proto(rename_all = PascalCase)]
pub struct SessionData {
    #[proto(strong_none)]
    pub artist: Option<String>,
    #[proto(strong_none)]
    pub title: Option<String>,
    #[proto(transform = Self::duration_into_ms)]
    pub progress: Duration,
    #[proto(strong_none, transform = Self::duration_into_ms)]
    pub duration: Option<Duration>,
    pub playing: bool,
    pub rate: f64,
}

impl SessionData {
    fn duration_into_ms(duration: Duration) -> u64 {
        duration.as_millis() as u64
    }
}
