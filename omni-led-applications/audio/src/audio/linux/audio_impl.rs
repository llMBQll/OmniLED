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

use pulse::context::subscribe::{Facility, InterestMaskSet};
use pulse::context::{Context, FlagSet};
use pulse::mainloop::threaded::Mainloop;
use pulse::proplist::{Proplist, properties};
use pulse::volume::Volume;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use tokio::runtime::Handle;
use tokio::sync::mpsc::Sender;

use crate::AudioData;

pub struct AudioImpl {
    _main_loop: Mainloop,
    _ctx: Rc<RefCell<Context>>,
}

impl AudioImpl {
    pub fn new(tx: Sender<AudioData>, handle: Handle) -> Self {
        /**********************|
        | Create the main loop |
        |**********************/
        let mut proplist = Proplist::new().unwrap();
        proplist
            .set_str(properties::APPLICATION_NAME, "Audio")
            .unwrap();

        let mut main_loop = Mainloop::new().unwrap();
        let ctx = Rc::new(RefCell::new(
            Context::new_with_proplist(&main_loop, "AudioContext", &proplist).unwrap(),
        ));

        /*********************************************************|
        | Connect to the server and wait for it ot be initialized |
        |*********************************************************/
        ctx.borrow_mut()
            .connect(None, FlagSet::NOFLAGS, None)
            .unwrap();

        main_loop.start().unwrap();

        loop {
            match ctx.borrow_mut().get_state() {
                pulse::context::State::Ready => {
                    break;
                }
                pulse::context::State::Failed | pulse::context::State::Terminated => {
                    panic!("Libpulse startup failed");
                }
                _ => {
                    main_loop.wait();
                }
            }
        }

        /*******************|
        | Set initial state |
        |*******************/
        let current_index = Rc::new(RefCell::new(Option::<u32>::None));
        let current_state = Rc::new(RefCell::new((false, 0)));

        Self::update_default_sink(
            ctx.clone(),
            current_index.clone(),
            current_state.clone(),
            tx.clone(),
            handle.clone(),
        );

        /**************************|
        | Register event callbacks |
        |**************************/
        ctx.borrow_mut().set_subscribe_callback(Some(Box::new({
            let ctx = ctx.clone();
            move |facility, _, index| match facility {
                Some(Facility::Sink) => {
                    let introspector = ctx.borrow_mut().introspect();
                    introspector.get_sink_info_by_index(index, {
                        let tx = tx.clone();
                        let handle = handle.clone();
                        let current_index = current_index.clone();
                        let current_state = current_state.clone();
                        move |list| match list {
                            pulse::callbacks::ListResult::Item(info) => {
                                if !Self::is_current_index(current_index.clone(), info.index) {
                                    return;
                                }

                                let volume = Self::normalize_volume(info.volume.get()[0]);
                                let muted = info.mute;

                                Self::update_state(
                                    current_state.clone(),
                                    (muted, volume),
                                    tx.clone(),
                                    handle.clone(),
                                );
                            }
                            _ => {}
                        }
                    });
                }
                Some(Facility::Client) => Self::update_default_sink(
                    ctx.clone(),
                    current_index.clone(),
                    current_state.clone(),
                    tx.clone(),
                    handle.clone(),
                ),
                _ => {}
            }
        })));

        ctx.borrow_mut()
            .subscribe(InterestMaskSet::CLIENT | InterestMaskSet::SINK, |success| {
                assert!(success, "'subscribe' failed");
            });

        Self {
            _main_loop: main_loop,
            _ctx: ctx,
        }
    }

    fn normalize_volume(volume: Volume) -> i32 {
        const NORMAL: f32 = Volume::NORMAL.0 as f32;

        match volume.is_valid() {
            true => {
                let volume = volume.0 as f32 * 100f32 / NORMAL;
                volume.round() as i32
            }
            false => 0,
        }
    }

    fn is_current_index(current_index: Rc<RefCell<Option<u32>>>, index: u32) -> bool {
        let current = current_index.borrow_mut();
        let current = current.deref();
        match current {
            Some(current) => index == *current,
            None => false,
        }
    }

    fn update_state(
        current_state: Rc<RefCell<(bool, i32)>>,
        state: (bool, i32),
        tx: Sender<AudioData>,
        handle: Handle,
    ) {
        let mut current_state = current_state.borrow_mut();
        if *current_state == state {
            return;
        }
        *current_state = state;

        let (muted, volume) = state;
        handle.spawn(async move {
            tx.send(AudioData::new(muted, volume, None)).await.unwrap();
        });
    }

    fn update_default_sink(
        ctx: Rc<RefCell<Context>>,
        current_index: Rc<RefCell<Option<u32>>>,
        current_state: Rc<RefCell<(bool, i32)>>,
        tx: Sender<AudioData>,
        handle: Handle,
    ) {
        let introspector = ctx.borrow_mut().introspect();
        introspector.get_server_info(move |server_info| {
            let introspector = ctx.borrow_mut().introspect();
            let name = server_info.default_sink_name.as_ref().unwrap().to_string();

            introspector.get_sink_info_by_name(name.as_str(), {
                let current_index = current_index.clone();
                let current_state = current_state.clone();
                let tx = tx.clone();
                let handle = handle.clone();
                move |list| match list {
                    pulse::callbacks::ListResult::Item(info) => {
                        let tx = tx.clone();
                        if Self::is_current_index(current_index.clone(), info.index) {
                            return;
                        }

                        let volume = Self::normalize_volume(info.volume.get()[0]);
                        let muted = info.mute;
                        let name = info.description.as_ref().unwrap().to_string();

                        *current_index.borrow_mut() = Some(info.index);
                        *current_state.borrow_mut() = (muted, volume);
                        handle.spawn(async move {
                            tx.send(AudioData::new(muted, volume, Some(name)))
                                .await
                                .unwrap();
                        });
                    }
                    _ => {}
                }
            });
        });
    }
}
