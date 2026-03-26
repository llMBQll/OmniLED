use futures_util::StreamExt;
use log::error;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::time::interval;
use zbus::{Connection, fdo::DBusProxy, proxy, zvariant::OwnedValue};

use crate::{Data, media::session_data::SessionData};

#[derive(Debug)]
enum MprisEvent {
    PlayerAdded(String),
    PlayerRemoved(String),
    FullUpdate((String, SessionData)),
    MetadataUpdate((String, Metadata)),
    ProgressUpdate((String, Duration)),
    PlayingUpdate((String, bool)),
    RateUpdate((String, f64)),
    Tick,
}

pub struct MediaImpl {
    tx: Sender<Data>,
}

impl MediaImpl {
    pub fn new(tx: Sender<Data>) -> Self {
        Self { tx }
    }

    pub async fn run(&self) {
        let conn = Connection::session()
            .await
            .expect("Failed to connect to D-Bus session bus");

        let (event_tx, event_rx) = mpsc::channel(256);

        tokio::task::spawn(Self::mpris_event_loop(
            conn.clone(),
            event_tx.clone(),
            event_rx,
            self.tx.clone(),
        ));
        tokio::task::spawn({
            let event_tx = event_tx.clone();
            async move {
                let mut interval = interval(Duration::from_millis(500));
                loop {
                    let _ = interval.tick().await;
                    let _ = event_tx.send(MprisEvent::Tick).await;
                }
            }
        });
        Self::track_open_players(conn, event_tx).await;
    }

    async fn mpris_event_loop(
        conn: Connection,
        tx: Sender<MprisEvent>,
        mut rx: Receiver<MprisEvent>,
        data_tx: Sender<Data>,
    ) {
        let mut players = HashMap::<String, PlayerData>::new();

        while let Some(event) = rx.recv().await {
            match event {
                MprisEvent::PlayerAdded(name) => {
                    if players.contains_key(&name) {
                        continue;
                    }

                    let handle = tokio::task::spawn(Self::handle_player_updates(
                        conn.clone(),
                        name.clone(),
                        tx.clone(),
                    ));
                    let data = SessionData::default();

                    players.insert(
                        name,
                        PlayerData {
                            handle,
                            data,
                            last_update: Instant::now(),
                        },
                    );
                }
                MprisEvent::PlayerRemoved(name) => {
                    if let Some(player) = players.remove(&name) {
                        player.handle.abort();
                    }
                }
                MprisEvent::FullUpdate((name, data)) => {
                    if let Some(entry) = players.get_mut(&name) {
                        entry.data = data;
                        entry.last_update = Instant::now();
                        data_tx
                            .send((false, name, entry.data.clone()))
                            .await
                            .unwrap();
                    }
                }
                MprisEvent::MetadataUpdate((name, metadata)) => {
                    if let Some(entry) = players.get_mut(&name) {
                        Self::update_progress(entry);

                        entry.data.artist = metadata.artist;
                        entry.data.title = metadata.title;
                        entry.data.duration = metadata.duration;
                        data_tx
                            .send((false, name, entry.data.clone()))
                            .await
                            .unwrap();
                    }
                }
                MprisEvent::ProgressUpdate((name, progress)) => {
                    if let Some(entry) = players.get_mut(&name) {
                        entry.data.progress = progress;
                        entry.last_update = Instant::now();
                        data_tx
                            .send((false, name, entry.data.clone()))
                            .await
                            .unwrap();
                    }
                }
                MprisEvent::PlayingUpdate((name, playing)) => {
                    if let Some(entry) = players.get_mut(&name) {
                        Self::update_progress(entry);

                        entry.data.playing = playing;
                        data_tx
                            .send((false, name, entry.data.clone()))
                            .await
                            .unwrap();
                    }
                }
                MprisEvent::RateUpdate((name, rate)) => {
                    if let Some(entry) = players.get_mut(&name) {
                        // Update the progress with previous playback rate before setting the new one
                        Self::update_progress(entry);

                        entry.data.rate = rate;
                        data_tx
                            .send((false, name, entry.data.clone()))
                            .await
                            .unwrap();
                    }
                }
                MprisEvent::Tick => {
                    for (name, entry) in players.iter_mut() {
                        if !entry.data.playing {
                            continue;
                        }

                        Self::update_progress(entry);
                        data_tx
                            .send((false, name.clone(), entry.data.clone()))
                            .await
                            .unwrap();
                    }
                }
            }
        }
    }

    async fn handle_player_updates(conn: Connection, player_name: String, tx: Sender<MprisEvent>) {
        let proxy = match MprisPlayerProxy::builder(&conn)
            .destination(player_name.to_string())
            .expect("invalid destination")
            .path(MPRIS_OBJECT_PATH)
            .expect("invalid path")
            .build()
            .await
        {
            Ok(proxy) => proxy,
            Err(err) => {
                error!("Failed to build proxy for {player_name}: {err}");
                return;
            }
        };

        let mut metadata_stream = proxy.receive_metadata_changed().await;
        let mut rate_stream = proxy.receive_rate_changed().await;
        let mut seeked_stream = match proxy.receive_seeked().await {
            Ok(seeked_stream) => seeked_stream,
            Err(err) => {
                error!("Seeked stream failed for {player_name}: {err}");
                return;
            }
        };
        let mut status_stream = proxy.receive_playback_status_changed().await;

        // Read full session data before entering the loop with partial updates
        let _ = tx
            .send(MprisEvent::FullUpdate((
                player_name.clone(),
                Self::read_session_data(&proxy).await,
            )))
            .await;

        loop {
            tokio::select! {
                Some(metadata) = metadata_stream.next() => {
                    if let Ok(metadata) = metadata.get().await {
                        let metadata = Self::read_metadata(&metadata);
                        let _ = tx.send(MprisEvent::MetadataUpdate((player_name.clone(), metadata))).await;
                    }
                }
                Some(rate) = rate_stream.next() => {
                    if let Ok(rate) = rate.get().await {
                        let _ = tx.send(MprisEvent::RateUpdate((player_name.clone(), rate))).await;
                    }
                }
                Some(position) = seeked_stream.next() => {
                    if let Ok(args) = position.args() {
                        let progress = Duration::from_micros(args.position as u64);
                        let _ = tx.send(MprisEvent::ProgressUpdate((player_name.clone(), progress))).await;
                    }
                }
                Some(status) = status_stream.next() => {
                    if let Ok(status) = status.get().await {
                        let playing = status == "Playing";
                        let _ = tx.send(MprisEvent::PlayingUpdate((player_name.clone(), playing))).await;
                    }
                }
            }
        }
    }

    async fn track_open_players(conn: Connection, tx: Sender<MprisEvent>) {
        let dbus = DBusProxy::new(&conn)
            .await
            .expect("Failed to create org.freedesktop.DBus proxy");

        let mut name_changes = dbus
            .receive_name_owner_changed()
            .await
            .expect("Failed to subscribe to NameOwnerChanged");

        if let Ok(names) = dbus.list_names().await {
            for name in names {
                if name.starts_with(MPRIS_PREFIX) {
                    let _ = tx.send(MprisEvent::PlayerAdded(name.to_string())).await;
                }
            }
        }

        while let Some(signal) = name_changes.next().await {
            let Ok(args) = signal.args() else { continue };

            let name = args.name();
            if !name.starts_with(MPRIS_PREFIX) {
                continue;
            }

            let old = args.old_owner();
            let new = args.new_owner();

            match (old.as_deref(), new.as_deref()) {
                (Some(old), None) => {
                    let _ = tx.send(MprisEvent::PlayerRemoved(old.to_string())).await;
                }
                (None, Some(new)) => {
                    let _ = tx.send(MprisEvent::PlayerAdded(new.to_string())).await;
                }
                _ => continue,
            }
        }
    }

    async fn read_session_data(proxy: &MprisPlayerProxy<'_>) -> SessionData {
        let (metadata, position, rate, status) = tokio::join!(
            proxy.metadata(),
            proxy.position(),
            proxy.rate(),
            proxy.playback_status(),
        );

        let metadata = Self::read_metadata(&metadata.unwrap_or_default());
        let playing = status.unwrap_or_default() == "Playing";
        let progress = Duration::from_micros(position.unwrap_or(0) as u64);
        let rate = rate.unwrap_or(1.0);

        SessionData {
            artist: metadata.artist,
            title: metadata.title,
            progress,
            duration: metadata.duration,
            playing,
            rate,
        }
    }

    fn read_metadata(metadata: &HashMap<String, OwnedValue>) -> Metadata {
        Metadata {
            title: Self::read_title(metadata),
            artist: Self::read_artist(metadata),
            duration: Self::read_duration(metadata),
        }
    }

    fn read_title<'a>(metadata: &'a HashMap<String, OwnedValue>) -> String {
        metadata
            .get("xesam:title")
            .map(|val| val.downcast_ref::<String>().unwrap())
            .unwrap_or_default()
    }

    fn read_artist(metadata: &HashMap<String, OwnedValue>) -> String {
        metadata
            .get("xesam:artist")
            .map::<String, _>(|val| {
                let arr = val.downcast_ref::<zbus::zvariant::Array>().unwrap();
                arr.iter()
                    .map(|v| v.downcast_ref::<String>().unwrap())
                    .next()
                    .unwrap_or_default()
            })
            .unwrap_or_default()
    }

    fn read_duration(metadata: &HashMap<String, OwnedValue>) -> Duration {
        let length = metadata
            .get("mpris:length")
            .and_then(|v| Some(v.downcast_ref::<u64>().unwrap()))
            .unwrap_or_default();
        Duration::from_micros(length)
    }

    fn update_progress(entry: &mut PlayerData) {
        if entry.data.playing {
            let elapsed = entry.last_update.elapsed();
            entry.data.progress += elapsed.mul_f64(entry.data.rate);
        }
        entry.last_update = Instant::now();
    }
}

const MPRIS_PREFIX: &str = "org.mpris.MediaPlayer2.";
const MPRIS_OBJECT_PATH: &str = "/org/mpris/MediaPlayer2";

#[proxy(
    interface = "org.mpris.MediaPlayer2.Player",
    default_path = "/org/mpris/MediaPlayer2",
    default_service = "org.mpris.MediaPlayer2"
)]
trait MprisPlayer {
    #[zbus(property)]
    fn playback_status(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn metadata(&self) -> zbus::Result<HashMap<String, OwnedValue>>;

    #[zbus(property)]
    fn position(&self) -> zbus::Result<i64>;

    #[zbus(property)]
    fn rate(&self) -> zbus::Result<f64>;

    #[zbus(signal)]
    fn seeked(&self, position: i64) -> zbus::Result<()>;
}

#[derive(Debug)]
struct PlayerData {
    handle: tokio::task::JoinHandle<()>,
    data: SessionData,
    last_update: Instant,
}

#[derive(Debug)]
struct Metadata {
    title: String,
    artist: String,
    duration: Duration,
}
