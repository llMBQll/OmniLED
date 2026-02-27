use log::warn;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus as PlaybackStatus,
};

use crate::Data;
use crate::media::session_data::SessionData;
use crate::media::windows::global_system_media::{GlobalSystemMedia, Message};

pub struct MediaImpl {
    tx: Sender<Data>,
}

struct SessionState {
    data: SessionData,
    last_update: Instant,
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
        let mut sessions: HashMap<String, SessionState> = HashMap::new();
        let mut current_session: Option<String> = None;

        let mut heartbeat = tokio::time::interval(Duration::from_secs(1));

        loop {
            tokio::select! {
                message = rx.recv() => {
                    match message {
                        Some(Message::SessionAdded(session)) => {
                            let name = Self::get_name(&session);
                            let (artist, title) = Self::get_song(&session).await;
                            let (progress, duration) = Self::get_progress(&session);
                            let (playing, rate) = Self::playback_info(&session);

                            sessions.insert(
                                name,
                                SessionState {
                                    data: SessionData {
                                        artist,
                                        title,
                                        progress,
                                        duration,
                                        playing,
                                        rate,
                                    },
                                    last_update: Instant::now(),
                                },
                            );
                        }
                        Some(Message::SessionRemoved(session)) => {
                            let name = Self::get_name(&session);
                            sessions.remove(&name);
                        }
                        Some(Message::CurrentSessionChanged(session)) => {
                            current_session = match session {
                                Some(session) => Some(Self::get_name(&session)),
                                None => None,
                            };
                        }
                        Some(Message::PlaybackInfoChanged(session)) => {
                            let name = Self::get_name(&session);

                            if let Some(state) = sessions.get_mut(&name) {
                                let (playing, rate) = Self::playback_info(&session);
                                state.data.playing = playing;
                                state.data.rate = rate;
                                state.last_update = Instant::now();

                                Self::send_data(&tx, name, state.data.clone(), &current_session).await;
                            }
                        }
                        Some(Message::MediaPropertiesChanged(session)) => {
                            let name = Self::get_name(&session);

                            if let Some(state) = sessions.get_mut(&name) {
                                let (artist, title) = Self::get_song(&session).await;
                                state.data.artist = artist;
                                state.data.title = title;

                                Self::send_data(&tx, name, state.data.clone(), &current_session).await;
                            }
                        }
                        Some(Message::TimelinePropertiesChanged(session)) => {
                            let name = Self::get_name(&session);

                            if let Some(state) = sessions.get_mut(&name) {
                                let (progress, duration) = Self::get_progress(&session);
                                state.data.progress = progress;
                                state.data.duration = duration;
                                state.last_update = Instant::now();

                                Self::send_data(&tx, name, state.data.clone(), &current_session).await;
                            }
                        }
                        None => break,
                    }
                }

                _ = heartbeat.tick() => {
                    for (name, state) in &sessions {
                        if state.data.playing && state.data.rate > 0.0 {
                            let elapsed = state.last_update.elapsed();
                            let progress_delta = elapsed.mul_f64(state.data.rate);

                            let mut data = state.data.clone();

                            data.progress = (data.progress + progress_delta)
                                .min(state.data.duration);

                            Self::send_data(&tx, name.clone(), data, &current_session).await;
                        }
                    }
                }
            }
        }
    }

    fn get_name(session: &GlobalSystemMediaTransportControlsSession) -> String {
        session.SourceAppUserModelId().unwrap().to_string_lossy()
    }

    async fn get_song(session: &GlobalSystemMediaTransportControlsSession) -> (String, String) {
        const DEFAULT_STR: &str = "N/A";

        match session.TryGetMediaPropertiesAsync() {
            Ok(operation) => match operation.await {
                Ok(properties) => Ok((
                    properties
                        .Artist()
                        .and_then(|x| Ok(x.to_string_lossy()))
                        .unwrap_or(DEFAULT_STR.to_string()),
                    properties
                        .Title()
                        .and_then(|x| Ok(x.to_string_lossy()))
                        .unwrap_or(DEFAULT_STR.to_string()),
                )),
                Err(err) => Err(err),
            },
            Err(err) => Err(err),
        }
        .unwrap_or_else(|err| {
            warn!("{err}");
            (DEFAULT_STR.to_string(), DEFAULT_STR.to_string())
        })
    }

    fn get_progress(session: &GlobalSystemMediaTransportControlsSession) -> (Duration, Duration) {
        const DEFAULT_POSITION: Duration = Duration::from_millis(0);
        const DEFAULT_END: Duration = Duration::from_millis(1);

        match session.GetTimelineProperties() {
            Ok(properties) => (
                properties
                    .Position()
                    .and_then(|x| Ok(x.into()))
                    .unwrap_or(DEFAULT_POSITION),
                properties
                    .EndTime()
                    .and_then(|x| Ok(x.into()))
                    .unwrap_or(DEFAULT_END),
            ),
            Err(err) => {
                warn!("{err}");
                (DEFAULT_POSITION, DEFAULT_END)
            }
        }
    }

    fn playback_info(session: &GlobalSystemMediaTransportControlsSession) -> (bool, f64) {
        const DEFAULT_STATUS: bool = false;
        const DEFAULT_RATE: f64 = 1.0;

        match session.GetPlaybackInfo() {
            Ok(info) => (
                info.PlaybackStatus()
                    .and_then(|x| Ok(x == PlaybackStatus::Playing))
                    .unwrap_or(DEFAULT_STATUS),
                info.PlaybackRate()
                    .and_then(|x| x.Value())
                    .unwrap_or(DEFAULT_RATE),
            ),
            Err(err) => {
                warn!("{err}");
                (DEFAULT_STATUS, DEFAULT_RATE)
            }
        }
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
