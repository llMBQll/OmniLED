use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use log::Level;
use log4rs::filter::{Filter, Response};

use crate::logging::logger::LevelFilter;

#[derive(Debug, Clone)]
pub struct DynamicFilter {
    filter_map: Arc<RwLock<HashMap<String, LevelFilter>>>,
}

impl DynamicFilter {
    pub fn new(filter_map: HashMap<String, LevelFilter>) -> Self {
        Self {
            filter_map: Arc::new(RwLock::new(filter_map)),
        }
    }

    pub fn set(&self, filter_map: HashMap<String, LevelFilter>) {
        *self.filter_map.write().unwrap() = filter_map;
    }

    #[inline]
    fn select_response(target_level: Level, level_filter: LevelFilter) -> Response {
        if level_filter < target_level {
            return Response::Reject;
        } else {
            return Response::Accept;
        }
    }

    fn respond(&self, mut target: &str, target_level: Level) -> Response {
        let filter_map = self.filter_map.read().unwrap();

        loop {
            match filter_map.get(target) {
                Some(level_filter) => {
                    return Self::select_response(target_level, *level_filter);
                }
                None => match target.rfind("::") {
                    Some(index) => {
                        target = &target[..index];
                    }
                    None => {
                        break;
                    }
                },
            }
        }

        // Only allow error logging if target is not registered above
        Self::select_response(target_level, LevelFilter::Error)
    }
}

impl Filter for DynamicFilter {
    fn filter(&self, record: &log::Record) -> Response {
        Self::respond(&self, record.target(), record.level())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_matching() {
        let f = DynamicFilter::new(HashMap::from([
            (String::from("a"), LevelFilter::Info),
            (String::from("b"), LevelFilter::Off),
        ]));

        // Matches "a" >= Info
        assert_eq!(f.respond("a", Level::Trace), Response::Reject);
        assert_eq!(f.respond("a", Level::Debug), Response::Reject);
        assert_eq!(f.respond("a", Level::Info), Response::Accept);
        assert_eq!(f.respond("a", Level::Warn), Response::Accept);
        assert_eq!(f.respond("a", Level::Error), Response::Accept);

        // Matches "b" >= Off
        assert_eq!(f.respond("b", Level::Trace), Response::Reject);
        assert_eq!(f.respond("b", Level::Debug), Response::Reject);
        assert_eq!(f.respond("b", Level::Info), Response::Reject);
        assert_eq!(f.respond("b", Level::Warn), Response::Reject);
        assert_eq!(f.respond("b", Level::Error), Response::Reject);
    }

    #[test]
    fn specific_target_matching() {
        let f = DynamicFilter::new(HashMap::from([
            (String::from("a"), LevelFilter::Info),
            (String::from("a::b"), LevelFilter::Warn),
            (String::from("a::b::c"), LevelFilter::Error),
        ]));

        // Matches "a::b::c" >= Error
        assert_eq!(f.respond("a::b::c::extra", Level::Error), Response::Accept);
        assert_eq!(f.respond("a::b::c::extra", Level::Warn), Response::Reject);
        assert_eq!(f.respond("a::b::c", Level::Error), Response::Accept);
        assert_eq!(f.respond("a::b::c", Level::Warn), Response::Reject);

        // Matches "a::b" >= Warn
        assert_eq!(f.respond("a::b::extra", Level::Warn), Response::Accept);
        assert_eq!(f.respond("a::b::extra", Level::Info), Response::Reject);
        assert_eq!(f.respond("a::b", Level::Warn), Response::Accept);
        assert_eq!(f.respond("a::b", Level::Info), Response::Reject);

        // Matches "a" >= Warn
        assert_eq!(f.respond("a::extra", Level::Info), Response::Accept);
        assert_eq!(f.respond("a::extra", Level::Debug), Response::Reject);
        assert_eq!(f.respond("a", Level::Info), Response::Accept);
        assert_eq!(f.respond("a", Level::Debug), Response::Reject);
    }

    #[test]
    fn no_matching_target() {
        let f = DynamicFilter::new(HashMap::from([]));

        // Expect that no matching target will be only allowed to log errors
        assert_eq!(f.respond("a", Level::Trace), Response::Reject);
        assert_eq!(f.respond("a", Level::Debug), Response::Reject);
        assert_eq!(f.respond("a", Level::Info), Response::Reject);
        assert_eq!(f.respond("a", Level::Warn), Response::Reject);
        assert_eq!(f.respond("a", Level::Error), Response::Accept);
    }
}
