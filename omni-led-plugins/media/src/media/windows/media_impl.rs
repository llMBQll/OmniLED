use log::warn;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus as PlaybackStatus,
};
use windows::core::HSTRING;

use crate::Data;
use crate::media::session_data::SessionData;
use crate::media::windows::global_system_media::{GlobalSystemMedia, Message};

pub struct MediaImpl;

struct SessionState {
    data: SessionData,
    last_update: Instant,
}

impl MediaImpl {
    pub async fn run(audio_tx: Sender<Data>) {
        let (tx, rx): (Sender<Message>, Receiver<Message>) = mpsc::channel(256);

        let loop_handle = tokio::task::spawn(async move {
            Self::run_message_loop(audio_tx, rx).await;
        });

        GlobalSystemMedia::init(tx).await;

        loop_handle.await.unwrap();
    }

    async fn run_message_loop(tx: Sender<Data>, mut rx: Receiver<Message>) {
        macro_rules! update_and_send {
            ($tx:expr, $current:expr, $name:expr, $entry:ident, $entry_update:block) => {
                Self::update_position($entry);
                $entry_update;
                Self::send_data($tx, $name, $entry.data.clone(), $current).await;
            };
        }

        let mut sessions: HashMap<String, SessionState> = HashMap::new();
        let mut current_session: Option<String> = None;

        while let Some(message) = rx.recv().await {
            match message {
                Message::SessionAdded(session) => {
                    let name = Self::get_name(&session);
                    let (artist, title) = Self::get_song(&session).await;
                    let (position, duration) = Self::get_position(&session);
                    let (playing, rate) = Self::playback_info(&session);

                    sessions.insert(
                        name,
                        SessionState {
                            data: SessionData {
                                artist,
                                title,
                                position,
                                duration,
                                playing,
                                rate,
                            },
                            last_update: Instant::now(),
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

                    if let Some(state) = sessions.get_mut(&name) {
                        let (playing, rate) = Self::playback_info(&session);

                        update_and_send!(&tx, &current_session, name, state, {
                            state.data.playing = playing;
                            state.data.rate = rate;
                            state.last_update = Instant::now();
                        });
                    }
                }
                Message::MediaPropertiesChanged(session) => {
                    let name = Self::get_name(&session);

                    if let Some(state) = sessions.get_mut(&name) {
                        let (artist, title) = Self::get_song(&session).await;

                        update_and_send!(&tx, &current_session, name, state, {
                            state.data.artist = artist;
                            state.data.title = title;
                        });
                    }
                }
                Message::TimelinePropertiesChanged(session) => {
                    let name = Self::get_name(&session);

                    if let Some(state) = sessions.get_mut(&name) {
                        let (position, duration) = Self::get_position(&session);

                        update_and_send!(&tx, &current_session, name, state, {
                            state.data.position = position;
                            state.data.duration = duration;
                        });
                    }
                }
                Message::Tick => {
                    for (name, state) in sessions.iter_mut() {
                        if state.data.playing {
                            update_and_send!(&tx, &current_session, name.clone(), state, {});
                        }
                    }
                }
            }
        }
    }

    fn get_name(session: &GlobalSystemMediaTransportControlsSession) -> String {
        session.SourceAppUserModelId().unwrap().to_string_lossy()
    }

    async fn get_song(
        session: &GlobalSystemMediaTransportControlsSession,
    ) -> (Option<String>, Option<String>) {
        let convert_string = |string: Result<HSTRING, _>| match string {
            Ok(string) if !string.is_empty() => Some(string.to_string_lossy()),
            _ => None,
        };

        match session.TryGetMediaPropertiesAsync() {
            Ok(operation) => match operation.await {
                Ok(properties) => Ok((
                    convert_string(properties.Artist()),
                    convert_string(properties.Title()),
                )),
                Err(err) => Err(err),
            },
            Err(err) => Err(err),
        }
        .unwrap_or_else(|err| {
            warn!("{err}");
            (None, None)
        })
    }

    fn get_position(
        session: &GlobalSystemMediaTransportControlsSession,
    ) -> (Duration, Option<Duration>) {
        const DEFAULT_POSITION: Duration = Duration::from_millis(0);

        match session.GetTimelineProperties() {
            Ok(properties) => (
                properties
                    .Position()
                    .and_then(|x| Ok(x.into()))
                    .unwrap_or(DEFAULT_POSITION),
                properties
                    .EndTime()
                    .and_then(|x| match Duration::from(x) {
                        Duration::ZERO => Ok(None),
                        x => Ok(Some(x)),
                    })
                    .unwrap_or(None),
            ),
            Err(err) => {
                warn!("{err}");
                (DEFAULT_POSITION, None)
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

    fn update_position(entry: &mut SessionState) {
        let now = Instant::now();
        if entry.data.playing {
            let elapsed = now.saturating_duration_since(entry.last_update);
            entry.data.position += elapsed.mul_f64(entry.data.rate);
        }
        entry.last_update = now;
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
