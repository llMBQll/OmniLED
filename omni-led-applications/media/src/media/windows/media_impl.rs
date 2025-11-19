use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use windows::Foundation::TimeSpan;
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus,
};

use crate::Data;
use crate::media::session_data::SessionData;
use crate::media::windows::global_system_media::{GlobalSystemMedia, Message};

pub struct MediaImpl {
    tx: Sender<Data>,
}

impl MediaImpl {
    pub fn new(tx: Sender<Data>) -> Self {
        Self { tx }
    }

    pub async fn run(&self) {
        let (tx, rx): (Sender<Message>, Receiver<Message>) = mpsc::channel(256);

        let audio_tx = self.tx.clone();
        let loop_handle = tokio::task::spawn(async move {
            Self::run_message_loop(audio_tx, rx).await;
        });

        GlobalSystemMedia::init(tx).await;

        loop_handle.await.unwrap();
    }

    async fn run_message_loop(tx: Sender<Data>, mut rx: Receiver<Message>) {
        let mut sessions: HashMap<String, SessionData> = HashMap::new();
        let mut current_session: Option<String> = None;
        while let Some(message) = rx.recv().await {
            match message {
                Message::SessionAdded(session) => {
                    let name = Self::get_name(&session);
                    let (artist, title) = Self::get_song(&session);
                    let (progress, duration) = Self::get_progress(&session);
                    let playing = Self::is_playing(&session);

                    sessions.insert(
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
                Message::SessionRemoved(session) => {
                    let name = Self::get_name(&session);

                    sessions.remove(&name);
                }
                Message::CurrentSessionChanged(session) => {
                    current_session = match session {
                        Some(session) => Some(Self::get_name(&session)),
                        None => None,
                    };
                }
                Message::PlaybackInfoChanged(session) => {
                    let name = Self::get_name(&session);

                    match sessions.get_mut(&name) {
                        Some(entry) => {
                            entry.playing = Self::is_playing(&session);

                            Self::send_data(&tx, name, entry.clone(), &current_session).await;
                        }
                        None => {}
                    }
                }
                Message::MediaPropertiesChanged(session) => {
                    let name = Self::get_name(&session);

                    match sessions.get_mut(&name) {
                        Some(entry) => {
                            let (artist, title) = Self::get_song(&session);
                            entry.artist = artist;
                            entry.title = title;

                            Self::send_data(&tx, name, entry.clone(), &current_session).await;
                        }
                        None => {}
                    }
                }
                Message::TimelinePropertiesChanged(session) => {
                    let name = Self::get_name(&session);

                    match sessions.get_mut(&name) {
                        Some(entry) => {
                            let (progress, duration) = Self::get_progress(&session);
                            entry.progress = progress;
                            entry.duration = duration;

                            Self::send_data(&tx, name, entry.clone(), &current_session).await;
                        }
                        None => {}
                    }
                }
            }
        }
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

    fn is_playing(session: &GlobalSystemMediaTransportControlsSession) -> bool {
        let info = session.GetPlaybackInfo().unwrap();
        let playing = info.PlaybackStatus().unwrap()
            == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing;

        playing
    }

    fn is_current(name: &String, current: &Option<String>) -> bool {
        match current {
            Some(current_name) => *name == *current_name,
            None => false,
        }
    }

    async fn send_data(
        tx: &Sender<Data>,
        name: String,
        data: SessionData,
        current: &Option<String>,
    ) {
        let is_current = Self::is_current(&name, current);
        if data.playing {
            tx.send((is_current, name, data)).await.unwrap();
        }
    }
}
