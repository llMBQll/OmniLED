use pulse::context::introspect::ServerInfo;
use pulse::context::subscribe::{Facility, InterestMaskSet};
use pulse::context::{Context, FlagSet};
use pulse::mainloop::threaded::Mainloop;
use pulse::proplist::{Proplist, properties};
use pulse::volume::Volume;
use std::cell::RefCell;
use std::rc::Rc;
use tokio::runtime::Handle;
use tokio::sync::mpsc::Sender;

use crate::{DeviceData, DeviceType};

#[derive(Default)]
struct Devices {
    input: DeviceState,
    output: DeviceState,
}

#[derive(Default)]
struct DeviceState {
    index: Option<u32>,
    muted: bool,
    volume: i32,
}

pub struct AudioImpl {
    _main_loop: Mainloop,
    _ctx: Rc<RefCell<Context>>,
}

impl AudioImpl {
    pub fn new(tx: Sender<(DeviceData, DeviceType)>, handle: Handle) -> Self {
        /**********************|
        | Create the main loop |
        |**********************/
        let mut properties = Proplist::new().unwrap();
        properties
            .set_str(properties::APPLICATION_NAME, "Audio")
            .unwrap();

        let mut main_loop = Mainloop::new().unwrap();
        let ctx = Rc::new(RefCell::new(
            Context::new_with_proplist(&main_loop, "AudioContext", &properties).unwrap(),
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
        let devices = Rc::new(RefCell::new(Devices::default()));

        Self::update_devices(ctx.clone(), devices.clone(), tx.clone(), handle.clone());

        /**************************|
        | Register event callbacks |
        |**************************/
        ctx.borrow_mut().set_subscribe_callback(Some(Box::new({
            macro_rules! update_device_info {
                ($devices:ident, $device_type:expr, $tx:ident, $handle:ident) => {{
                    let tx = $tx.clone();
                    let handle = $handle.clone();
                    let devices = $devices.clone();
                    move |list| match list {
                        pulse::callbacks::ListResult::Item(info) => {
                            let device = match $device_type {
                                DeviceType::Input => &mut devices.borrow_mut().input,
                                DeviceType::Output => &mut devices.borrow_mut().output,
                            };

                            if !Self::is_current_index(device, info.index) {
                                return;
                            }

                            let volume = Self::normalize_volume(info.volume.get()[0]);
                            let muted = info.mute;
                            Self::update_state(
                                device,
                                muted,
                                volume,
                                $device_type,
                                tx.clone(),
                                handle.clone(),
                            );
                        }
                        _ => {}
                    }
                }};
            }

            let ctx = ctx.clone();
            move |facility, _op, index| match facility {
                Some(Facility::Sink) => {
                    let introspector = ctx.borrow_mut().introspect();
                    introspector.get_sink_info_by_index(index, {
                        update_device_info!(devices, DeviceType::Output, tx, handle)
                    });
                }
                Some(Facility::Source) => {
                    let introspector = ctx.borrow_mut().introspect();
                    introspector.get_source_info_by_index(index, {
                        update_device_info!(devices, DeviceType::Input, tx, handle)
                    });
                }
                Some(Facility::Server) => {
                    Self::update_devices(ctx.clone(), devices.clone(), tx.clone(), handle.clone());
                }
                _ => {}
            }
        })));

        ctx.borrow_mut().subscribe(
            InterestMaskSet::SINK | InterestMaskSet::SOURCE | InterestMaskSet::SERVER,
            |success| {
                assert!(success, "'subscribe' failed");
            },
        );

        Self {
            _main_loop: main_loop,
            _ctx: ctx,
        }
    }

    fn normalize_volume(volume: Volume) -> i32 {
        const NORMAL: f32 = Volume::NORMAL.0 as f32;

        match volume.is_valid() {
            true => {
                let volume = volume.0 as f32 * 100.0 / NORMAL;
                volume.round() as i32
            }
            false => 0,
        }
    }

    fn is_current_index(device: &DeviceState, index: u32) -> bool {
        match device.index {
            Some(current) => current == index,
            None => false,
        }
    }

    fn update_state(
        device: &mut DeviceState,
        muted: bool,
        volume: i32,
        device_type: DeviceType,
        tx: Sender<(DeviceData, DeviceType)>,
        handle: Handle,
    ) {
        if device.muted == muted && device.volume == volume {
            return;
        }
        device.muted = muted;
        device.volume = volume;

        handle.spawn(async move {
            tx.send((DeviceData::new(true, muted, volume, None), device_type))
                .await
                .unwrap();
        });
    }

    fn get_device_name(server_info: &ServerInfo, device_type: DeviceType) -> String {
        let name = match device_type {
            DeviceType::Input => &server_info.default_source_name,
            DeviceType::Output => &server_info.default_sink_name,
        };
        name.as_ref().unwrap().to_string()
    }

    fn update_devices(
        ctx: Rc<RefCell<Context>>,
        devices: Rc<RefCell<Devices>>,
        tx: Sender<(DeviceData, DeviceType)>,
        handle: Handle,
    ) {
        macro_rules! get_device_info {
            ($ctx:ident, $devices:ident, $device_type:expr, $tx:ident, $handle:ident) => {{
                let devices = $devices.clone();
                let tx = $tx.clone();
                let handle = $handle.clone();
                move |list| match list {
                    pulse::callbacks::ListResult::Item(info) => {
                        let device = match $device_type {
                            DeviceType::Input => &mut devices.borrow_mut().input,
                            DeviceType::Output => &mut devices.borrow_mut().output,
                        };

                        if Self::is_current_index(device, info.index) {
                            return;
                        }

                        let volume = Self::normalize_volume(info.volume.get()[0]);
                        let muted = info.mute;
                        let name = info.description.as_ref().unwrap().to_string();

                        let tx = tx.clone();
                        device.index = Some(info.index);
                        device.muted = muted;
                        device.volume = volume;
                        handle.spawn(async move {
                            tx.send((
                                DeviceData::new(true, muted, volume, Some(name)),
                                $device_type,
                            ))
                            .await
                            .unwrap();
                        });
                    }
                    _ => {}
                }
            }};
        }

        let introspector = ctx.borrow_mut().introspect();
        introspector.get_server_info(move |server_info| {
            let introspector = ctx.borrow_mut().introspect();

            let name = Self::get_device_name(&server_info, DeviceType::Input);
            introspector.get_source_info_by_name(name.as_str(), {
                get_device_info!(ctx, devices, DeviceType::Input, tx, handle)
            });

            let name = Self::get_device_name(&server_info, DeviceType::Output);
            introspector.get_sink_info_by_name(name.as_str(), {
                get_device_info!(ctx, devices, DeviceType::Output, tx, handle)
            });
        });
    }
}
