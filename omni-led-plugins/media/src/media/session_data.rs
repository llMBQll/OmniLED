use serde::{Serialize, Serializer};
use std::time::Duration;

#[derive(Serialize, Clone, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct SessionData {
    pub artist: Option<String>,
    pub title: Option<String>,
    #[serde(serialize_with = "duration_ms")]
    pub position: Duration,
    #[serde(serialize_with = "option_duration_ms")]
    pub duration: Option<Duration>,
    pub playing: bool,
    pub rate: f64,
}

fn duration_ms<S>(duration: &Duration, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_u64(duration.as_millis() as u64)
}

fn option_duration_ms<S>(duration: &Option<Duration>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match duration {
        Some(duration) => s.serialize_u64(duration.as_millis() as u64),
        None => s.serialize_none(),
    }
}
