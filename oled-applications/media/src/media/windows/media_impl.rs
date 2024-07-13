use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::runtime::Handle;
use tokio::sync::mpsc::Sender;
use windows::Foundation::TimeSpan;
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus,
};

use crate::media::session_data::SessionData;
use crate::media::windows::global_system_media::GlobalSystemMedia;
use crate::Message;

pub struct MediaImpl {
    system_media: GlobalSystemMedia,
    sessions: Arc<Mutex<HashMap<String, SessionData>>>,
    current_session: Arc<Mutex<Option<String>>>,
}

impl MediaImpl {
    pub fn new(tx: Sender<Message>, handle: Handle) -> Self {
        let mut media = Self {
            system_media: GlobalSystemMedia::new(),
            sessions: Arc::new(Mutex::new(HashMap::new())),
            current_session: Arc::new(Mutex::new(None)),
        };

        media.setup(tx, handle);

        media
    }

    pub async fn run(&self) {
        tokio::time::sleep(Duration::MAX).await;
    }

    fn setup(&mut self, tx: Sender<Message>, handle: Handle) {
        let tx = Arc::new(tx);

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
            let tx = Arc::clone(&tx);
            let handle = handle.clone();
            move |(_, session)| {
                let name = Self::get_name(&session);

                let mut guard = sessions.lock().unwrap();
                match guard.get_mut(&name) {
                    Some(entry) => {
                        entry.playing = Self::get_status(&session);

                        Self::send_data(
                            tx.clone(),
                            handle.clone(),
                            name,
                            entry.clone(),
                            &current_session,
                        );
                    }
                    None => {}
                }
            }
        });

        self.system_media.register_on_media_properties_changed({
            let sessions = Arc::clone(&self.sessions);
            let current_session = Arc::clone(&self.current_session);
            let tx = Arc::clone(&tx);
            let handle = handle.clone();
            move |(_, session)| {
                let name = Self::get_name(&session);

                let mut guard = sessions.lock().unwrap();
                match guard.get_mut(&name) {
                    Some(entry) => {
                        let (artist, title) = Self::get_song(&session);
                        entry.artist = artist;
                        entry.title = title;

                        Self::send_data(
                            tx.clone(),
                            handle.clone(),
                            name,
                            entry.clone(),
                            &current_session,
                        );
                    }
                    None => {}
                }
            }
        });

        self.system_media.register_on_timeline_properties_changed({
            let sessions = Arc::clone(&self.sessions);
            let current_session = Arc::clone(&self.current_session);
            let tx = Arc::clone(&tx);
            move |(_, session)| {
                let name = Self::get_name(&session);

                let mut guard = sessions.lock().unwrap();
                match guard.get_mut(&name) {
                    Some(entry) => {
                        let (progress, duration) = Self::get_progress(&session);
                        entry.progress = progress;
                        entry.duration = duration;

                        Self::send_data(
                            tx.clone(),
                            handle.clone(),
                            name,
                            entry.clone(),
                            &current_session,
                        );
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
        let properties = session.TryGetMediaPropertiesAsync().unwrap().get().unwrap();
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

    fn send_data(
        tx: Arc<Sender<Message>>,
        handle: Handle,
        name: String,
        data: SessionData,
        current: &Arc<Mutex<Option<String>>>,
    ) {
        let is_current = Self::is_current(&name, current);
        if data.playing {
            handle.spawn(async move {
                tx.send(Message::Event(is_current, name, data))
                    .await
                    .unwrap();
            });
        }
    }
}
