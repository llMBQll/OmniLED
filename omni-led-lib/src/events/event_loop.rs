use log::trace;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use crate::events::event_queue::{Event, EventQueue};
use crate::script_handler::script_data_types::DurationWrapper;
use crate::settings::settings_value::SettingsValue;

pub struct EventLoop {}

impl EventLoop {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run<F: FnMut(Duration, Vec<Event>)>(
        &self,
        interval_value: SettingsValue<DurationWrapper>,
        running: &AtomicBool,
        mut handler: F,
    ) {
        let mut interval = Duration::ZERO;
        while running.load(Ordering::Relaxed) {
            let begin = Instant::now();

            let event_queue = EventQueue::instance();
            let events = event_queue.lock().unwrap().get_events();

            handler(interval, events);

            let end = Instant::now();
            let update_duration = end - begin;
            trace!("Update took {:?}", update_duration);

            interval = interval_value.get().0;
            std::thread::sleep(interval.saturating_sub(update_duration));
        }
    }
}
