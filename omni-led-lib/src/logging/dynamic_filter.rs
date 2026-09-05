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
    fn respond(target_level: Level, level_filter: LevelFilter) -> Response {
        if level_filter < target_level {
            return Response::Reject;
        } else {
            return Response::Accept;
        }
    }
}

impl Filter for DynamicFilter {
    fn filter(&self, record: &log::Record) -> Response {
        let filter_map = self.filter_map.read().unwrap();

        let mut target = record.target();
        loop {
            match filter_map.get(target) {
                Some(level_filter) => {
                    return Self::respond(record.level(), *level_filter);
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
        return Self::respond(record.level(), LevelFilter::Error);
    }
}
