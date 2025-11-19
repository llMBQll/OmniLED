use omni_led_derive::IntoProto;
use std::time::Duration;

#[derive(Clone, IntoProto)]
#[proto(rename_all = PascalCase)]
pub struct SessionData {
    pub artist: String,
    pub title: String,
    #[proto(transform = Self::duration_into_ms)]
    pub progress: Duration,
    #[proto(transform = Self::duration_into_ms)]
    pub duration: Duration,
    pub playing: bool,
}

impl SessionData {
    fn duration_into_ms(duration: Duration) -> i64 {
        duration.as_millis() as i64
    }
}
