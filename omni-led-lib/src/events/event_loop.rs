use log::trace;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use crate::events::event_queue::{Event, EventQueue};
use crate::script_handler::script_data_types::DurationWrapper;

pub struct EventLoop {}

impl EventLoop {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run<F: FnMut(Duration, Vec<Event>)>(
        &self,
        interval: Rc<RefCell<DurationWrapper>>,
        running: &AtomicBool,
        mut handler: F,
    ) {
        while running.load(Ordering::Relaxed) {
            let begin = Instant::now();
            let interval = (*interval.borrow()).0;

            let event_queue = EventQueue::instance();
            let events = event_queue.lock().unwrap().get_events();

            handler(interval, events);

            let end = Instant::now();
            let update_duration = end - begin;
            trace!("Update took {:?}", update_duration);

            std::thread::sleep(interval.saturating_sub(update_duration));
        }
    }
}
