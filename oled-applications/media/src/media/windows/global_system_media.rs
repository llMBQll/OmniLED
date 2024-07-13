use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use windows::Media::Control::{
    CurrentSessionChangedEventArgs, MediaPropertiesChangedEventArgs, PlaybackInfoChangedEventArgs,
    TimelinePropertiesChangedEventArgs,
};
use windows::{
    Foundation::TypedEventHandler,
    Media::Control::{
        GlobalSystemMediaTransportControlsSession,
        GlobalSystemMediaTransportControlsSessionManager, SessionsChangedEventArgs,
    },
};

type MediaEvent = Event<MediaEventData>;

type MediaOptionalEvent = Event<MediaEventOptionalData>;

type MediaEventData = (
    GlobalSystemMediaTransportControlsSessionManager,
    GlobalSystemMediaTransportControlsSession,
);

type MediaEventOptionalData = (
    GlobalSystemMediaTransportControlsSessionManager,
    Option<GlobalSystemMediaTransportControlsSession>,
);

pub struct GlobalSystemMedia {
    manager: GlobalSystemMediaTransportControlsSessionManager,
    sessions: Arc<Mutex<HashMap<String, GlobalSystemMediaTransportControlsSession>>>,

    on_session_added: Arc<Mutex<MediaEvent>>,
    on_session_removed: Arc<Mutex<MediaEvent>>,
    on_current_session_changed: Arc<Mutex<MediaOptionalEvent>>,
    on_playback_info_changed: Arc<Mutex<MediaEvent>>,
    on_media_properties_changed: Arc<Mutex<MediaEvent>>,
    on_timeline_properties_changed: Arc<Mutex<MediaEvent>>,
}

impl GlobalSystemMedia {
    pub fn new() -> Self {
        let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
            .unwrap()
            .get()
            .unwrap();

        Self {
            manager,
            sessions: Arc::new(Mutex::new(HashMap::new())),
            on_session_added: Arc::new(Mutex::new(Event::new())),
            on_session_removed: Arc::new(Mutex::new(Event::new())),
            on_current_session_changed: Arc::new(Mutex::new(Event::new())),
            on_playback_info_changed: Arc::new(Mutex::new(Event::new())),
            on_media_properties_changed: Arc::new(Mutex::new(Event::new())),
            on_timeline_properties_changed: Arc::new(Mutex::new(Event::new())),
        }
    }

    pub fn start(&self) {
        let sessions = self.manager.GetSessions().unwrap();

        *self.sessions.lock().unwrap() = HashMap::from_iter(
            sessions
                .into_iter()
                .map(|session| (session.SourceAppUserModelId().unwrap().to_string(), session)),
        );

        for (_, session) in self.sessions.lock().unwrap().iter() {
            self.on_session_added
                .lock()
                .unwrap()
                .fire((self.manager.clone(), session.clone()));

            Self::register_session_handlers(
                &self.manager,
                &session,
                &self.on_playback_info_changed,
                &self.on_media_properties_changed,
                &self.on_timeline_properties_changed,
            );
        }

        self.register_global_handlers();
    }

    pub fn register_on_session_added<T: FnMut(MediaEventData) + Send + 'static>(
        &mut self,
        func: T,
    ) -> usize {
        self.on_session_added.lock().unwrap().connect(func)
    }

    pub fn register_on_session_removed<T: FnMut(MediaEventData) + Send + 'static>(
        &mut self,
        func: T,
    ) -> usize {
        self.on_session_removed.lock().unwrap().connect(func)
    }

    pub fn register_on_current_session_changed<
        T: FnMut(MediaEventOptionalData) + Send + 'static,
    >(
        &mut self,
        func: T,
    ) -> usize {
        self.on_current_session_changed
            .lock()
            .unwrap()
            .connect(func)
    }

    pub fn register_on_playback_info_changed<T: FnMut(MediaEventData) + Send + 'static>(
        &mut self,
        func: T,
    ) -> usize {
        self.on_playback_info_changed.lock().unwrap().connect(func)
    }

    pub fn register_on_media_properties_changed<T: FnMut(MediaEventData) + Send + 'static>(
        &mut self,
        func: T,
    ) -> usize {
        self.on_media_properties_changed
            .lock()
            .unwrap()
            .connect(func)
    }

    pub fn register_on_timeline_properties_changed<T: FnMut(MediaEventData) + Send + 'static>(
        &mut self,
        func: T,
    ) -> usize {
        self.on_timeline_properties_changed
            .lock()
            .unwrap()
            .connect(func)
    }

    fn register_global_handlers(&self) {
        self.manager
            .CurrentSessionChanged(&TypedEventHandler::new({
                let manager = self.manager.clone();
                let handler = Arc::clone(&self.on_current_session_changed);
                move |_manager: &Option<GlobalSystemMediaTransportControlsSessionManager>,
                      _args: &Option<CurrentSessionChangedEventArgs>| {
                    let session = match manager.GetCurrentSession() {
                        Ok(session) => Some(session),
                        Err(_) => None,
                    };
                    handler.lock().unwrap().fire((manager.clone(), session));
                    Ok(())
                }
            }))
            .unwrap();

        self.manager
            .SessionsChanged(&TypedEventHandler::new({
                let manager = self.manager.clone();
                let sessions = Arc::clone(&self.sessions);
                let on_session_added = Arc::clone(&self.on_session_added);
                let on_session_removed = Arc::clone(&self.on_session_removed);
                let on_playback_info_changed = Arc::clone(&self.on_playback_info_changed);
                let on_media_properties_changed = Arc::clone(&self.on_media_properties_changed);
                let on_timeline_properties_changed =
                    Arc::clone(&self.on_timeline_properties_changed);
                move |_manager: &Option<GlobalSystemMediaTransportControlsSessionManager>,
                      _args: &Option<SessionsChangedEventArgs>| {
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
                                on_session_removed
                                    .lock()
                                    .unwrap()
                                    .fire((manager.clone(), session));
                            }
                            None => {}
                        }
                    }

                    let new_sessions: Vec<_> = incoming_sessions
                        .into_iter()
                        .filter(|(name, _)| !sessions.contains_key(name))
                        .collect();

                    for (name, session) in new_sessions {
                        Self::register_session_handlers(
                            &manager,
                            &session,
                            &on_playback_info_changed,
                            &on_media_properties_changed,
                            &on_timeline_properties_changed,
                        );

                        on_session_added
                            .lock()
                            .unwrap()
                            .fire((manager.clone(), session.clone()));

                        sessions.insert(name, session);
                    }

                    Ok(())
                }
            }))
            .unwrap();
    }

    fn register_session_handlers(
        manager: &GlobalSystemMediaTransportControlsSessionManager,
        session: &GlobalSystemMediaTransportControlsSession,
        on_playback_info_changed: &Arc<Mutex<MediaEvent>>,
        on_media_properties_changed: &Arc<Mutex<MediaEvent>>,
        on_timeline_properties_changed: &Arc<Mutex<MediaEvent>>,
    ) {
        session
            .PlaybackInfoChanged(&TypedEventHandler::new({
                let manager = manager.clone();
                let session = session.clone();
                let handler = Arc::clone(&on_playback_info_changed);
                move |_: &Option<GlobalSystemMediaTransportControlsSession>,
                      _: &Option<PlaybackInfoChangedEventArgs>| {
                    handler
                        .lock()
                        .unwrap()
                        .fire((manager.clone(), session.clone()));
                    Ok(())
                }
            }))
            .unwrap();

        session
            .MediaPropertiesChanged(&TypedEventHandler::new({
                let manager = manager.clone();
                let session = session.clone();
                let handler = Arc::clone(&on_media_properties_changed);
                move |_: &Option<GlobalSystemMediaTransportControlsSession>,
                      _: &Option<MediaPropertiesChangedEventArgs>| {
                    handler
                        .lock()
                        .unwrap()
                        .fire((manager.clone(), session.clone()));
                    Ok(())
                }
            }))
            .unwrap();

        session
            .TimelinePropertiesChanged(&TypedEventHandler::new({
                let manager = manager.clone();
                let session = session.clone();
                let handler = Arc::clone(&on_timeline_properties_changed);
                move |_: &Option<GlobalSystemMediaTransportControlsSession>,
                      _: &Option<TimelinePropertiesChangedEventArgs>| {
                    handler
                        .lock()
                        .unwrap()
                        .fire((manager.clone(), session.clone()));
                    Ok(())
                }
            }))
            .unwrap();
    }
}

struct Event<Args: Clone, Ret = ()> {
    id: usize,
    listeners: Vec<(usize, Box<dyn FnMut(Args) -> Ret + Send>)>,
}

impl<Args: Clone, Ret> Event<Args, Ret> {
    pub fn new() -> Self {
        Self {
            id: 0,
            listeners: Vec::new(),
        }
    }

    pub fn connect<T: FnMut(Args) -> Ret + Send + 'static>(&mut self, func: T) -> usize {
        let id = self.id;
        self.id += 1;

        self.listeners.push((id, Box::new(func)));

        id
    }

    #[allow(unused)]
    pub fn disconnect(&mut self, subscription_id: usize) {
        self.listeners.retain(|(id, _)| *id != subscription_id)
    }

    pub fn fire(&mut self, args: Args) {
        for (_id, listener) in self.listeners.iter_mut() {
            listener(args.clone());
        }
    }
}
