use media_remote::{NowPlayingInfo, NowPlayingPerl, Subscription};
use std::sync::RwLockReadGuard;
use std::time::Duration;
use tokio::sync::mpsc::Sender;

use crate::{Data, media::session_data::SessionData};

pub struct MediaImpl;

impl MediaImpl {
    pub async fn run(tx: Sender<Data>) {
        let now_playing = NowPlayingPerl::new();

        now_playing.subscribe({
            let tx = tx.clone();
            move |guard| {
                send_data(&tx, guard);
            }
        });

        // TODO track current position
        tokio::time::sleep(Duration::MAX).await;
    }
}

fn convert_duration(secs: f64) -> Duration {
    let ms = (secs * 1000.0).round() as u64;
    Duration::from_millis(ms)
}

fn send_data(tx: &Sender<Data>, guard: RwLockReadGuard<'_, Option<NowPlayingInfo>>) {
    let info = guard.as_ref();
    if let Some(info) = info {
        let name = match &info.bundle_name {
            Some(name) => name.clone(),
            None => return,
        };

        let data = SessionData {
            artist: info.artist.clone(),
            title: info.title.clone(),
            position: convert_duration(info.elapsed_time.unwrap_or(0.0)),
            duration: info.duration.and_then(|x| Some(convert_duration(x))),
            playing: info.is_playing.unwrap_or(false),
            rate: info.playback_rate.unwrap_or(1.0),
        };

        // TODO convert to tx.send and make the function async
        _ = tx.try_send((true, name, data));
    }
}
