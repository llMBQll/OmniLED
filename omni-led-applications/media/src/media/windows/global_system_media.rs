/*
 * OmniLED is a software for displaying data on various OLED devices.
 * Copyright (C) 2024  Michał Bałabanow <m.balabanow@gmail.com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::runtime::Handle;
use tokio::sync::mpsc::Sender;
use windows::{
    Foundation::TypedEventHandler,
    Media::Control::{
        CurrentSessionChangedEventArgs, GlobalSystemMediaTransportControlsSession,
        GlobalSystemMediaTransportControlsSessionManager, MediaPropertiesChangedEventArgs,
        PlaybackInfoChangedEventArgs, SessionsChangedEventArgs, TimelinePropertiesChangedEventArgs,
    },
    core::Ref,
};

type MediaEventData = GlobalSystemMediaTransportControlsSession;
type MediaEventOptionalData = Option<GlobalSystemMediaTransportControlsSession>;

pub enum Message {
    SessionAdded(MediaEventData),
    SessionRemoved(MediaEventData),
    CurrentSessionChanged(MediaEventOptionalData),
    PlaybackInfoChanged(MediaEventData),
    MediaPropertiesChanged(MediaEventData),
    TimelinePropertiesChanged(MediaEventData),
}

pub struct GlobalSystemMedia;

impl GlobalSystemMedia {
    pub async fn init(tx: Sender<Message>) {
        let handle = Handle::current();
        let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
            .unwrap()
            .get()
            .unwrap();
        let sessions = manager.GetSessions().unwrap();

        let sessions = HashMap::from_iter(
            sessions
                .into_iter()
                .map(|session| (session.SourceAppUserModelId().unwrap().to_string(), session)),
        );

        for (_, session) in sessions.iter() {
            tx.send(Message::SessionAdded(session.clone()))
                .await
                .unwrap();

            Self::register_session_handlers(&session, &tx, handle.clone());
        }

        let sessions = Arc::new(Mutex::new(sessions));
        Self::register_global_handlers(tx, handle, &manager, &sessions);
    }

    fn register_global_handlers(
        tx: Sender<Message>,
        handle: Handle,
        manager: &GlobalSystemMediaTransportControlsSessionManager,
        sessions: &Arc<Mutex<HashMap<String, GlobalSystemMediaTransportControlsSession>>>,
    ) {
        manager
            .CurrentSessionChanged(&TypedEventHandler::new({
                let manager = manager.clone();
                let tx = tx.clone();
                let handle = handle.clone();
                move |_manager: Ref<'_, GlobalSystemMediaTransportControlsSessionManager>,
                      _args: Ref<'_, CurrentSessionChangedEventArgs>| {
                    let session = match manager.GetCurrentSession() {
                        Ok(session) => Some(session),
                        Err(_) => None,
                    };
                    let tx = tx.clone();
                    handle.spawn(async move {
                        tx.send(Message::CurrentSessionChanged(session.clone()))
                            .await
                            .unwrap();
                    });
                    Ok(())
                }
            }))
            .unwrap();

        manager
            .SessionsChanged(&TypedEventHandler::new({
                let manager = manager.clone();
                let sessions = Arc::clone(&sessions);
                let tx = tx.clone();
                let handle = handle.clone();
                move |_manager: Ref<'_, GlobalSystemMediaTransportControlsSessionManager>,
                      _args: Ref<'_, SessionsChangedEventArgs>| {
                    let mut sessions = sessions.lock().unwrap();

                    let incoming_sessions = manager.GetSessions()?;
                    let incoming_sessions =
                        HashMap::<String, GlobalSystemMediaTransportControlsSession>::from_iter(
                            incoming_sessions.into_iter().map(|session| {
                                (session.SourceAppUserModelId().unwrap().to_string(), session)
                            }),
                        );

                    let to_remove: Vec<_> = sessions
                        .iter()
                        .filter_map(|(name, _)| match incoming_sessions.contains_key(name) {
                            true => None,
                            false => Some(name.clone()),
                        })
                        .collect();

                    for name in to_remove {
                        let session = sessions.remove(&name);
                        match session {
                            Some(session) => {
                                let tx = tx.clone();
                                handle.spawn(async move {
                                    tx.send(Message::SessionRemoved(session.clone()))
                                        .await
                                        .unwrap();
                                });
                            }
                            None => {}
                        }
                    }

                    let new_sessions: Vec<_> = incoming_sessions
                        .into_iter()
                        .filter(|(name, _)| !sessions.contains_key(name))
                        .collect();

                    for (name, session) in new_sessions {
                        Self::register_session_handlers(&session, &tx, handle.clone());

                        let tx = tx.clone();
                        handle.spawn({
                            let session = session.clone();
                            async move {
                                tx.send(Message::SessionAdded(session)).await.unwrap();
                            }
                        });

                        sessions.insert(name, session);
                    }

                    Ok(())
                }
            }))
            .unwrap();
    }

    fn register_session_handlers(
        session: &GlobalSystemMediaTransportControlsSession,
        tx: &Sender<Message>,
        handle: Handle,
    ) {
        session
            .PlaybackInfoChanged(&TypedEventHandler::new({
                let session = session.clone();
                let handle = handle.clone();
                let tx = tx.clone();
                move |_session: Ref<'_, GlobalSystemMediaTransportControlsSession>,
                      _args: Ref<'_, PlaybackInfoChangedEventArgs>| {
                    let session = session.clone();
                    let tx = tx.clone();
                    handle.spawn(async move {
                        tx.send(Message::PlaybackInfoChanged(session.clone()))
                            .await
                            .unwrap();
                    });

                    Ok(())
                }
            }))
            .unwrap();

        session
            .MediaPropertiesChanged(&TypedEventHandler::new({
                let session = session.clone();
                let handle = handle.clone();
                let tx = tx.clone();
                move |_session: Ref<'_, GlobalSystemMediaTransportControlsSession>,
                      _args: Ref<'_, MediaPropertiesChangedEventArgs>| {
                    let session = session.clone();
                    let tx = tx.clone();
                    handle.spawn(async move {
                        tx.send(Message::MediaPropertiesChanged(session.clone()))
                            .await
                            .unwrap();
                    });
                    Ok(())
                }
            }))
            .unwrap();

        session
            .TimelinePropertiesChanged(&TypedEventHandler::new({
                let session = session.clone();
                let handle = handle.clone();
                let tx = tx.clone();
                move |_session: Ref<'_, GlobalSystemMediaTransportControlsSession>,
                      _args: Ref<'_, TimelinePropertiesChangedEventArgs>| {
                    let session = session.clone();
                    let tx = tx.clone();
                    handle.spawn(async move {
                        tx.send(Message::TimelinePropertiesChanged(session.clone()))
                            .await
                            .unwrap();
                    });
                    Ok(())
                }
            }))
            .unwrap();
    }
}
