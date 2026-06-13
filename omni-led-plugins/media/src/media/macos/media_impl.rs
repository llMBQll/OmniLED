use media_remote::{NowPlayingPerl, Subscription};
use std::time::{Duration, Instant};
use tokio::runtime::Handle;
use tokio::sync::mpsc;
use tokio::time::MissedTickBehavior::Skip;
use tokio::time::interval;

use crate::Data;
use crate::media::session_data::SessionData;

pub struct MediaImpl;

impl MediaImpl {
    pub async fn run(tx: mpsc::Sender<Data>) {
        let now_playing = NowPlayingPerl::new();

        let (subscribe_tx, mut subscribe_rx) = mpsc::channel(100);

        now_playing.subscribe({
            let handle = Handle::current();
            move |guard| {
                let info = guard.as_ref();
                if let Some(info) = info {
                    let name = match &info.bundle_name {
                        Some(name) => name.clone(),
                        None => return,
                    };

                    // macOS reports the elapsed time and a timestamp of when was the update sent
                    // both times need to be added together to get current elapsed time
                    let elapsed_since_update =
                        info.info_update_time.map_or(Duration::ZERO, |update_time| {
                            update_time.elapsed().unwrap_or(Duration::ZERO)
                        });
                    let position = info.elapsed_time.map_or(Duration::ZERO, |elapsed_time| {
                        convert_duration(elapsed_time) + elapsed_since_update
                    });

                    let data = SessionData {
                        artist: info.artist.clone(),
                        title: info.title.clone(),
                        position,
                        duration: info.duration.and_then(|x| Some(convert_duration(x))),
                        playing: info.is_playing.unwrap_or(false),
                        rate: info.playback_rate.unwrap_or(1.0),
                    };

                    let subscribe_tx = subscribe_tx.clone();
                    handle.spawn(async move {
                        _ = subscribe_tx.send((name, data)).await;
                    });
                }
            }
        });

        let mut interval = interval(Duration::from_millis(500));
        interval.set_missed_tick_behavior(Skip);

        let mut last_update = Instant::now();
        let mut name = String::new();
        let mut data = SessionData::default();

        loop {
            tokio::select! {
                Some((new_name, new_data)) = subscribe_rx.recv() => {
                    last_update = Instant::now();
                    name = new_name;
                    data = new_data;

                    _ = tx.send((true, name.clone(), data.clone())).await;
                }
                _ = interval.tick() => {
                    if data.playing {
                        let now = Instant::now();
                        let elapsed = now.saturating_duration_since(last_update);
                        data.position += elapsed.mul_f64(data.rate);
                        last_update = now;

                        _ = tx.send((true, name.clone(), data.clone())).await;
                    }
                }
            }
        }
    }
}

fn convert_duration(secs: f64) -> Duration {
    let ms = (secs * 1000.0).round() as u64;
    Duration::from_millis(ms)
}
