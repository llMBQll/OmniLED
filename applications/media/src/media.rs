use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use windows::Foundation::TimeSpan;
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus,
};

use crate::global_system_media::GlobalSystemMedia;

pub struct Media {
    system_media: GlobalSystemMedia,
    sessions: Arc<Mutex<HashMap<String, SessionData>>>,
    current_session: Arc<Mutex<Option<String>>>,
}

impl Media {
    pub fn new() -> Self {
        let mut media = Self {
            system_media: GlobalSystemMedia::new(),
            sessions: Arc::new(Mutex::new(HashMap::new())),
            current_session: Arc::new(Mutex::new(None)),
        };

        media.setup();

        media
    }

    fn setup(&mut self) {
        self.system_media.register_on_session_added({
            let sessions = Arc::clone(&self.sessions);
            move |(_, session)| {
                let name = Self::get_name(&session);
                let (artist, title) = Self::get_song(&session);
                let (progress, duration) = Self::get_progress(&session);
                let playing = Self::get_status(&session);

                sessions.lock().unwrap().insert(
                    name,
                    SessionData {
                        artist,
                        title,
                        progress,
                        duration,
                        playing,
                    },
                );
            }
        });

        self.system_media.register_on_session_removed({
            let sessions = Arc::clone(&self.sessions);
            move |(_, session)| {
                let name = Self::get_name(&session);

                sessions.lock().unwrap().remove(&name);
            }
        });

        self.system_media.register_on_current_session_changed({
            let current_session = Arc::clone(&self.current_session);
            move |(_, session)| {
                *current_session.lock().unwrap() = match session {
                    Some(session) => Some(Self::get_name(&session)),
                    None => None,
                };
            }
        });

        self.system_media.register_on_playback_info_changed({
            let sessions = Arc::clone(&self.sessions);
            let current_session = Arc::clone(&self.current_session);
            move |(_, session)| {
                let name = Self::get_name(&session);

                let mut guard = sessions.lock().unwrap();
                match guard.get_mut(&name) {
                    Some(entry) => {
                        entry.playing = Self::get_status(&session);
                        println!("{:?}", entry);
                    }
                    None => {}
                }
            }
        });

        self.system_media.register_on_media_properties_changed({
            let sessions = Arc::clone(&self.sessions);
            let current_session = Arc::clone(&self.current_session);
            move |(_, session)| {
                let name = Self::get_name(&session);

                let mut guard = sessions.lock().unwrap();
                match guard.get_mut(&name) {
                    Some(entry) => {
                        let (artist, title) = Self::get_song(&session);
                        entry.artist = artist;
                        entry.title = title;
                        println!("{:?}", entry);
                    }
                    None => {}
                }
            }
        });

        self.system_media.register_on_timeline_properties_changed({
            let sessions = Arc::clone(&self.sessions);
            let current_session = Arc::clone(&self.current_session);
            move |(_, session)| {
                let name = Self::get_name(&session);

                let mut guard = sessions.lock().unwrap();
                match guard.get_mut(&name) {
                    Some(entry) => {
                        let (progress, duration) = Self::get_progress(&session);
                        entry.progress = progress;
                        entry.duration = duration;
                        println!("{:?}", entry);
                    }
                    None => {}
                }
            }
        });

        self.system_media.start();
    }

    fn get_name(session: &GlobalSystemMediaTransportControlsSession) -> String {
        session.SourceAppUserModelId().unwrap().to_string_lossy()
    }

    fn get_song(session: &GlobalSystemMediaTransportControlsSession) -> (String, String) {
        let properties = smol::block_on(session.TryGetMediaPropertiesAsync().unwrap()).unwrap();
        let artist = properties.Artist().unwrap().to_string_lossy();
        let title = properties.Title().unwrap().to_string_lossy();

        (artist, title)
    }

    fn get_progress(session: &GlobalSystemMediaTransportControlsSession) -> (Duration, Duration) {
        let to_duration = |timespan: TimeSpan| {
            let ms = timespan.Duration / 10000;
            Duration::from_millis(ms as u64)
        };

        let properties = session.GetTimelineProperties().unwrap();
        let progress = to_duration(properties.Position().unwrap());
        let duration = to_duration(properties.EndTime().unwrap());

        (progress, duration)
    }

    fn get_status(session: &GlobalSystemMediaTransportControlsSession) -> bool {
        let info = session.GetPlaybackInfo().unwrap();
        let playing = info.PlaybackStatus().unwrap()
            == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing;

        playing
    }

    fn is_current(name: &String, current: &Arc<Mutex<Option<String>>>) -> bool {
        match current.lock().unwrap().as_ref() {
            Some(current_name) => *name == *current_name,
            None => false,
        }
    }
}

#[derive(Debug)]
struct SessionData {
    artist: String,
    title: String,
    progress: Duration,
    duration: Duration,
    playing: bool,
}
