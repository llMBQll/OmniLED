use mpris::{DBusError, Player, PlayerFinder};
use std::collections::HashSet;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::media::session_data::SessionData;
use crate::Data;

pub struct MediaImpl {
    tx: Sender<Data>,
}

impl MediaImpl {
    pub fn new(tx: Sender<Data>) -> Self {
        Self { tx }
    }

    pub async fn run(&self) {
        let data_tx = self.tx.clone();

        let local = tokio::task::LocalSet::new();
        local
            .run_until(async move {
                let (tx, rx): (Sender<MediaMessage>, Receiver<MediaMessage>) = mpsc::channel(256);

                let loop_handle = tokio::task::spawn_local({
                    let tx = tx.clone();
                    async move {
                        Self::process_player_updates(data_tx, tx, rx).await;
                    }
                });

                Self::discover_players(tx).await;

                loop_handle.await.unwrap();
            })
            .await;
    }

    async fn discover_players(tx: Sender<MediaMessage>) {
        loop {
            let finder = PlayerFinder::new().expect("Could not connect to D-Bus");
            let players = finder.find_all().unwrap();

            for player in players {
                tx.send(MediaMessage::PlayerDiscovered(player))
                    .await
                    .unwrap();
            }

            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }

    async fn process_player_updates(
        data_tx: Sender<Data>,
        tx: Sender<MediaMessage>,
        mut rx: Receiver<MediaMessage>,
    ) {
        let mut active_players = HashSet::<String>::new();

        while let Some(message) = rx.recv().await {
            match message {
                MediaMessage::PlayerDiscovered(player) => {
                    let name = player.bus_name_player_name_part().to_string();
                    if !active_players.insert(name.to_string()) {
                        // Player already has a running event loop
                        continue;
                    }

                    let tx = tx.clone();
                    let data_tx = data_tx.clone();
                    tokio::task::spawn_local(async move {
                        Self::process_player(data_tx, name.clone(), player).await;

                        tx.send(MediaMessage::PlayerRemoved(name)).await.unwrap();
                    });
                }
                MediaMessage::PlayerRemoved(name) => {
                    active_players.remove(&name);
                }
            }
        }
    }

    async fn process_player(tx: Sender<Data>, name: String, player: Player) {
        loop {
            if !player.is_running() {
                break;
            }

            let playback_status = player.get_playback_status().unwrap();

            match playback_status {
                mpris::PlaybackStatus::Playing => {
                    let data = Self::get_session_data(&player).unwrap();
                    tx.send((false, name.clone(), data)).await.unwrap();
                }
                _ => {}
            }

            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    }

    fn get_session_data(player: &Player) -> Result<SessionData, DBusError> {
        let metadata = player.get_metadata()?;
        let artist = metadata.artists().unwrap_or(vec![""])[0];
        let title = metadata.title().unwrap_or_default();
        let progress = player.get_position().unwrap_or_default();
        let duration = metadata.length().unwrap_or_default();

        Ok(SessionData {
            artist: artist.to_string(),
            title: title.to_string(),
            progress,
            duration,
            playing: true,
        })
    }
}

enum MediaMessage {
    PlayerDiscovered(Player),
    PlayerRemoved(String),
}
