use futures_util::StreamExt;
use log::error;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::time::MissedTickBehavior::Skip;
use tokio::time::interval;
use zbus::{Connection, fdo::DBusProxy, proxy, zvariant::OwnedValue};

use crate::{Data, media::session_data::SessionData};

pub struct MediaImpl;

impl MediaImpl {
    pub async fn run(tx: Sender<Data>) {
        let conn = Connection::session()
            .await
            .expect("Failed to connect to D-Bus session bus");

        let (event_tx, event_rx) = mpsc::channel(256);

        tokio::task::spawn(Self::mpris_event_loop(
            conn.clone(),
            event_tx.clone(),
            event_rx,
            tx,
        ));
        tokio::task::spawn({
            let event_tx = event_tx.clone();
            async move {
                let mut interval = interval(Duration::from_millis(500));
                interval.set_missed_tick_behavior(Skip);
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
        macro_rules! update_and_send {
            ($tx:ident, $entry:ident, $entry_update:block) => {
                Self::update_position($entry);
                $entry_update;
                $tx.send((false, $entry.player_name.clone(), $entry.data.clone()))
                    .await
                    .unwrap();
            };
        }

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
                    let player_name = Self::get_player_name(&name);

                    players.insert(
                        name,
                        PlayerData {
                            handle,
                            data,
                            last_update: Instant::now(),
                            player_name,
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
                        update_and_send!(data_tx, entry, {
                            entry.data = data;
                        });
                    }
                }
                MprisEvent::MetadataUpdate((name, metadata)) => {
                    if let Some(entry) = players.get_mut(&name) {
                        update_and_send!(data_tx, entry, {
                            if entry.data.title != metadata.title
                                || entry.data.artist != metadata.artist
                            {
                                entry.data.position = Duration::ZERO;
                            }
                            entry.data.artist = metadata.artist;
                            entry.data.title = metadata.title;
                            entry.data.duration = metadata.duration;
                        });
                    }
                }
                MprisEvent::PositionUpdate((name, position)) => {
                    if let Some(entry) = players.get_mut(&name) {
                        update_and_send!(data_tx, entry, {
                            entry.data.position = position;
                        });
                    }
                }
                MprisEvent::PlayingUpdate((name, playing)) => {
                    if let Some(entry) = players.get_mut(&name) {
                        update_and_send!(data_tx, entry, {
                            entry.data.playing = playing;
                        });
                    }
                }
                MprisEvent::RateUpdate((name, rate)) => {
                    if let Some(entry) = players.get_mut(&name) {
                        update_and_send!(data_tx, entry, {
                            entry.data.rate = rate;
                        });
                    }
                }
                MprisEvent::Tick => {
                    for (_, entry) in players.iter_mut() {
                        if entry.data.playing {
                            update_and_send!(data_tx, entry, {});
                        }
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
                        let position = Duration::from_micros(args.position as u64);
                        let _ = tx.send(MprisEvent::PositionUpdate((player_name.clone(), position))).await;
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
                (Some(_), None) => {
                    let _ = tx.send(MprisEvent::PlayerRemoved(name.to_string())).await;
                }
                (None, Some(_)) => {
                    let _ = tx.send(MprisEvent::PlayerAdded(name.to_string())).await;
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
        let position = Duration::from_micros(position.unwrap_or(0) as u64);
        let rate = rate.unwrap_or(1.0);

        SessionData {
            artist: metadata.artist,
            title: metadata.title,
            position,
            duration: metadata.duration,
            playing,
            rate,
        }
    }

    fn read_metadata(metadata: &HashMap<String, OwnedValue>) -> Metadata {
        Metadata {
            artist: Self::read_artist(metadata),
            title: Self::read_title(metadata),
            duration: Self::read_duration(metadata),
        }
    }

    fn read_artist(metadata: &HashMap<String, OwnedValue>) -> Option<String> {
        let val = metadata.get("xesam:artist")?;
        let arr = val.downcast_ref::<zbus::zvariant::Array>().ok()?;
        match arr.iter().next() {
            Some(val) => match val.downcast_ref::<String>() {
                Ok(artist) if !artist.is_empty() => Some(artist),
                _ => None,
            },
            None => None,
        }
    }

    fn read_title(metadata: &HashMap<String, OwnedValue>) -> Option<String> {
        let val = metadata.get("xesam:title")?;
        match val.downcast_ref::<String>() {
            Ok(title) if !title.is_empty() => Some(title),
            _ => None,
        }
    }

    fn read_duration(metadata: &HashMap<String, OwnedValue>) -> Option<Duration> {
        let val = metadata.get("mpris:length")?;
        match val.downcast_ref::<u64>() {
            Ok(length) if length != 0 => Some(Duration::from_micros(length)),
            _ => None,
        }
    }

    fn update_position(entry: &mut PlayerData) {
        let now = Instant::now();
        if entry.data.playing {
            let elapsed = now.saturating_duration_since(entry.last_update);
            entry.data.position += elapsed.mul_f64(entry.data.rate);
        }
        entry.last_update = now;
    }

    fn get_player_name(name: &str) -> String {
        let name = name.strip_prefix(MPRIS_PREFIX).unwrap();

        match name.find('.') {
            Some(offset) => name[0..offset].to_string(),
            None => name.to_string(),
        }
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
enum MprisEvent {
    PlayerAdded(String),
    PlayerRemoved(String),
    FullUpdate((String, SessionData)),
    MetadataUpdate((String, Metadata)),
    PositionUpdate((String, Duration)),
    PlayingUpdate((String, bool)),
    RateUpdate((String, f64)),
    Tick,
}

#[derive(Debug)]
struct PlayerData {
    handle: tokio::task::JoinHandle<()>,
    data: SessionData,
    last_update: Instant,
    player_name: String,
}

#[derive(Debug)]
struct Metadata {
    artist: Option<String>,
    title: Option<String>,
    duration: Option<Duration>,
}
