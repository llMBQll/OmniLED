use std::time::Duration;

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct SessionData {
    pub artist: String,
    pub title: String,
    #[serde(serialize_with = "duration_to_ms")]
    pub progress: Duration,
    #[serde(serialize_with = "duration_to_ms")]
    pub duration: Duration,
    pub playing: bool,
}

fn duration_to_ms<S: serde::Serializer>(
    duration: &Duration,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_u128(duration.as_millis())
}
