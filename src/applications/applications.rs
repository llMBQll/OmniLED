use std::collections::hash_map::{DefaultHasher, Entry};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};
use mlua::{chunk, Lua, Nil, UserData, UserDataFields, UserDataMethods};
use tokio::time::{Duration, Instant};

pub struct Applications {
    applications: HashMap<u64, (String, Instant)>,
    timeout: Duration,
}

pub fn load_applications(lua: &Lua) -> Arc<Mutex<Applications>> {
    let applications = Arc::new(Mutex::new(Applications::new()));

    lua.globals().set("APPLICATIONS", Arc::clone(&applications)).unwrap();
    lua.load(chunk!{
        APPLICATIONS:set_timeout(SETTINGS["application_timeout"])
    }).exec().unwrap();

    applications
}

impl Applications {
    pub fn new() -> Self {
        Self {
            applications: HashMap::new(),
            timeout: Duration::ZERO,
        }
    }

    pub fn register(&mut self, name: &String) -> Option<u64> {
        let token = Self::hash(name);
        let now = Instant::now();
        let mut valid = false;

        self.applications.entry(token).and_modify(|(current, timeout)| {
            if now > *timeout {
                *current = name.clone();
                *timeout = now + self.timeout;
                valid = true;
            }
        }).or_insert_with(|| {
            valid = true;
            (name.clone(), now + self.timeout)
        });

        match valid {
            true => Some(token),
            false => None
        }
    }

    pub fn update(&mut self, token: u64) -> Option<(String, Instant)> {
        let now = Instant::now();
        let mut valid = false;

        let entry = self.applications.entry(token).and_modify(|(_, timeout)| {

            if now < *timeout {
                *timeout = now + self.timeout;
                valid = true;
            }
        });

        match entry {
            Entry::Occupied(x) => {
                if valid {
                    let (name, timeout) = x.get();
                    Some((name.clone(), timeout.clone()))
                } else {
                    None
                }
            }
            Entry::Vacant(_) => {
                None
            }
        }
    }

    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    fn hash<T: Hash>(t: &T) -> u64 {
        // TODO randomize hash on every startup

        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }
}

impl UserData for Applications {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("register", |_, state, name: String| {
            Ok(state.register(&name))
        });

        methods.add_method_mut("update", |_, state, token: u64| {
            match state.update(token) {
                Some(data) => Ok(Some(DataWrapper(data))),
                None => Ok(None)
            }
        });

        methods.add_method_mut("set_timeout", |_, state, timeout: u64| {
            Ok(state.set_timeout(Duration::from_millis(timeout)))
        });
    }
}

pub struct DataWrapper((String, Instant));

impl UserData for DataWrapper {
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("name", |_, data| {
            Ok(data.0.0.clone())
        });

        fields.add_field_method_get("timeout", |_, data| {
            Ok((data.0.1 - Instant::now()).as_millis() as u64)
        })
    }
}

