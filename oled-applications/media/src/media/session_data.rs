use oled_api::types::Table;
use std::time::Duration;

pub struct SessionData {
    pub artist: String,
    pub title: String,
    pub progress: Duration,
    pub duration: Duration,
    pub playing: bool,
}

impl Into<Table> for &SessionData {
    fn into(self) -> Table {
        let mut table = Table::default();

        table
            .items
            .insert("Artist".to_string(), self.artist.as_str().into());
        table
            .items
            .insert("Title".to_string(), self.title.as_str().into());
        table.items.insert(
            "Progress".to_string(),
            (self.progress.as_millis() as i64).into(),
        );
        table.items.insert(
            "Duration".to_string(),
            (self.duration.as_millis() as i64).into(),
        );
        table
            .items
            .insert("Playing".to_string(), self.playing.into());

        table
    }
}
