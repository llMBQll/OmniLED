use mlua::{Lua, MetaMethod, UserData};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{common::cleanup_guard::CleanupGuard, events::signal::Signal};

pub struct Events {
    signals: HashMap<String, Arc<Mutex<Signal>>>,
}

impl Events {
    pub fn load(lua: &Lua) -> CleanupGuard {
        let events = Events {
            signals: HashMap::new(),
        };
        lua.globals().set("EVENTS", events).unwrap();

        CleanupGuard::with_name(lua, "EVENTS")
    }

    pub fn get_or_create(&mut self, name: String) -> Arc<Mutex<Signal>> {
        let signal = self
            .signals
            .entry(name.clone())
            .or_insert(Arc::new(Mutex::new(Signal::new(name))));
        Arc::clone(signal)
    }

    pub fn get(&self, name: &String) -> Option<Arc<Mutex<Signal>>> {
        match self.signals.get(name) {
            Some(signal) => Some(Arc::clone(signal)),
            None => None,
        }
    }
}

impl UserData for Events {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("get_or_create", |_, this, name: String| {
            Ok(this.get_or_create(name))
        });

        methods.add_meta_method(MetaMethod::Index, |_, this, name: String| {
            Ok(this.get(&name))
        });
    }
}
