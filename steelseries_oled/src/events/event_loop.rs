use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use crate::events::event_queue::{Event, EventQueue};

pub struct EventLoop {}

impl EventLoop {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn run<F: FnMut(Vec<Event>)>(
        &self,
        interval: Duration,
        running: &AtomicBool,
        mut handler: F,
    ) {
        while running.load(Ordering::Relaxed) {
            let begin = Instant::now();

            let event_queue = EventQueue::instance();
            let events = event_queue.lock().unwrap().get_events();

            handler(events);

            let end = Instant::now();
            let update_duration = end - begin;
            // println!("Update took {} us", update_duration.as_micros());
            tokio::time::sleep(interval.saturating_sub(update_duration)).await;
        }
    }
}
