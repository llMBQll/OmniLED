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

use log::trace;
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
            trace!("Update took {:?}", update_duration);

            tokio::time::sleep(interval.saturating_sub(update_duration)).await;
        }
    }
}
