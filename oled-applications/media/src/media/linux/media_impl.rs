use mpris::PlayerFinder;
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use crate::media::session_data::SessionData;
use crate::media::Callback;

pub struct MediaImpl {
    callback: Arc<Mutex<Callback>>,
}

impl MediaImpl {
    pub fn new(callback: Arc<Mutex<Callback>>) -> Self {
        Self { callback }
    }

    pub async fn run(&self) {
        let callback = self.callback.clone();

        let local = tokio::task::LocalSet::new();
        local
            .run_until(async move {
                let active_players = Rc::new(RefCell::new(HashSet::<String>::new()));

                loop {
                    let finder = PlayerFinder::new().expect("Could not connect to D-Bus");
                    let players = finder.find_all().unwrap();

                    for player in players {
                        let active_players = Rc::clone(&active_players);
                        let callback = Arc::clone(&callback);
                        tokio::task::spawn_local(async move {
                            let name = player.bus_name_player_name_part();

                            match active_players.borrow_mut().insert(name.to_string()) {
                                true => {}
                                false => return,
                            }

                            loop {
                                if !player.is_running() {
                                    break;
                                }

                                let playback_status = match player.get_playback_status() {
                                    Ok(playback_status) => playback_status,
                                    Err(err) => {
                                        println!("{:?}", err);
                                        break;
                                    }
                                };

                                match playback_status {
                                    mpris::PlaybackStatus::Playing => {
                                        let metadata = player.get_metadata().unwrap();
                                        let artist = metadata.artists().unwrap_or(vec![""])[0];
                                        let title = metadata.title().unwrap_or_default();
                                        let progress = player.get_position().unwrap_or_default();
                                        let duration = metadata.length().unwrap_or_default();

                                        let data = SessionData {
                                            artist: artist.to_string(),
                                            title: title.to_string(),
                                            progress,
                                            duration,
                                            playing: true,
                                        };

                                        let name = name.to_string();
                                        callback.lock().unwrap()(&name, data, false);
                                    }
                                    _ => {}
                                }

                                tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                            }

                            active_players.borrow_mut().remove(name);
                        });
                    }

                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            })
            .await;
    }
}
