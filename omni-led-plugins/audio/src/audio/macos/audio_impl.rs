use objc2_core_audio::{
    AudioObjectAddPropertyListener, AudioObjectGetPropertyData, AudioObjectID,
    AudioObjectPropertyAddress, AudioObjectRemovePropertyListener,
};
use std::ffi::{CStr, c_void};
use std::mem::size_of;
use std::ptr::NonNull;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use crate::audio::macos::constants;
use crate::{DeviceData, DeviceType};

pub struct AudioImpl {
    ctx: Arc<Mutex<AudioContext>>,
}

struct AudioContext {
    pub tx: Sender<(DeviceData, DeviceType)>,
    pub current_ids: [AudioObjectID; 2],
}

const _: () = {
    assert!(DeviceType::Input as i32 == 0);
    assert!(DeviceType::Output as i32 == 1);
};

impl AudioImpl {
    pub fn new(tx: Sender<(DeviceData, DeviceType)>) -> Self {
        let current_ids = [
            get_default_device(DeviceType::Input),
            get_default_device(DeviceType::Output),
        ];

        let ctx = Arc::new(Mutex::new(AudioContext {
            tx: tx.clone(),
            current_ids,
        }));

        let ctx_ptr = Arc::as_ptr(&ctx) as *mut c_void;

        for device_type in [DeviceType::Input, DeviceType::Output] {
            register_system_listener(ctx_ptr, device_type);
            register_device_listeners(ctx_ptr, current_ids[device_type as usize], device_type);
            send_initial_device_update(current_ids[device_type as usize], &tx, device_type);
        }

        Self { ctx }
    }
}

impl Drop for AudioImpl {
    fn drop(&mut self) {
        let ctx_ptr = Arc::as_ptr(&self.ctx) as *mut c_void;
        let ctx = self.ctx.lock().unwrap();

        for device_type in [DeviceType::Input, DeviceType::Output] {
            unregister_system_listener(ctx_ptr, device_type);
            unregister_device_listeners(
                ctx_ptr,
                ctx.current_ids[device_type as usize],
                device_type,
            );
        }
    }
}

extern "C-unwind" fn system_listener(
    _in_object_id: AudioObjectID,
    _in_number_addresses: u32,
    _in_addresses: NonNull<AudioObjectPropertyAddress>,
    in_client_data: *mut c_void,
) -> i32 {
    let ctx = unsafe { &*(in_client_data as *const Mutex<AudioContext>) };
    if let Ok(mut ctx) = ctx.lock() {
        let addresses = unsafe {
            std::slice::from_raw_parts(_in_addresses.as_ptr(), _in_number_addresses as usize)
        };

        for addr in addresses {
            let device_type = match addr.mSelector {
                constants::DEFAULT_INPUT_DEVICE_SELECTOR => DeviceType::Input,
                constants::DEFAULT_OUTPUT_DEVICE_SELECTOR => DeviceType::Output,
                _ => continue,
            };

            let new_device_id = get_default_device(device_type);
            let current_id = &mut ctx.current_ids[device_type as usize];

            if new_device_id != *current_id {
                unregister_device_listeners(in_client_data, *current_id, device_type);

                *current_id = new_device_id;
                register_device_listeners(in_client_data, new_device_id, device_type);

                send_initial_device_update(new_device_id, &ctx.tx, device_type);
            }
        }
    }
    0
}

extern "C-unwind" fn device_listener(
    in_object_id: AudioObjectID,
    in_number_addresses: u32,
    in_addresses: NonNull<AudioObjectPropertyAddress>,
    in_client_data: *mut c_void,
) -> i32 {
    let context_mutex = unsafe { &*(in_client_data as *const Mutex<AudioContext>) };

    if let Ok(ctx) = context_mutex.lock() {
        let addresses = unsafe {
            std::slice::from_raw_parts(in_addresses.as_ptr(), in_number_addresses as usize)
        };

        let mut update_input = false;
        let mut update_output = false;
        for addr in addresses {
            let device_type = match addr.mScope {
                constants::INPUT_SCOPE => DeviceType::Input,
                constants::OUTPUT_SCOPE => DeviceType::Output,
                _ => continue,
            };

            if in_object_id == ctx.current_ids[device_type as usize]
                && (addr.mSelector == constants::VOLUME_SELECTOR
                    || addr.mSelector == constants::MUTE_SELECTOR)
            {
                update_input = device_type == DeviceType::Input;
                update_output = device_type == DeviceType::Output;
            }
        }

        if update_input {
            send_partial_device_update(in_object_id, &ctx.tx, DeviceType::Input);
        }
        if update_output {
            send_partial_device_update(in_object_id, &ctx.tx, DeviceType::Output);
        }
    }
    0
}

fn register_system_listener(ctx: *mut c_void, device_type: DeviceType) {
    unsafe {
        _ = AudioObjectAddPropertyListener(
            objc2_core_audio::kAudioObjectSystemObject as u32,
            NonNull::from_ref(constants::default_device_address(device_type)),
            Some(system_listener),
            ctx,
        );
    }
}

fn unregister_system_listener(ctx: *mut c_void, device_type: DeviceType) {
    unsafe {
        _ = AudioObjectRemovePropertyListener(
            objc2_core_audio::kAudioObjectSystemObject as u32,
            NonNull::from_ref(constants::default_device_address(device_type)),
            Some(system_listener),
            ctx,
        );
    }
}

fn register_device_listeners(ctx: *mut c_void, device_id: AudioObjectID, device_type: DeviceType) {
    if device_id == 0 {
        return;
    }

    for address in [
        constants::mute_address(device_type),
        constants::volume_address(device_type),
    ] {
        let _ = unsafe {
            AudioObjectAddPropertyListener(
                device_id,
                NonNull::from_ref(address),
                Some(device_listener),
                ctx,
            )
        };
    }
}

fn unregister_device_listeners(
    ctx: *mut c_void,
    device_id: AudioObjectID,
    device_type: DeviceType,
) {
    if device_id == 0 {
        return;
    }

    for address in [
        constants::mute_address(device_type),
        constants::volume_address(device_type),
    ] {
        let _ = unsafe {
            AudioObjectRemovePropertyListener(
                device_id,
                NonNull::from_ref(address),
                Some(device_listener),
                ctx,
            )
        };
    }
}

fn send_initial_device_update(
    device_id: AudioObjectID,
    tx: &Sender<(DeviceData, DeviceType)>,
    device_type: DeviceType,
) {
    if device_id == 0 {
        let _ = tx.send((DeviceData::new(false, true, 0, None), device_type));
    } else {
        let volume = get_device_volume(device_id, device_type);
        let mute = get_device_mute(device_id, device_type);
        let name = get_device_name(device_id, device_type);

        let _ = tx.send((DeviceData::new(true, mute, volume, Some(name)), device_type));
    }
}

fn send_partial_device_update(
    device_id: AudioObjectID,
    tx: &Sender<(DeviceData, DeviceType)>,
    device_type: DeviceType,
) {
    let volume = get_device_volume(device_id, device_type);
    let mute = get_device_mute(device_id, device_type);

    let _ = tx.send((DeviceData::new(true, mute, volume, None), device_type));
}

fn get_default_device(device_type: DeviceType) -> AudioObjectID {
    let mut device_id: AudioObjectID = 0;
    let size = size_of::<AudioObjectID>() as u32;

    unsafe {
        _ = AudioObjectGetPropertyData(
            objc2_core_audio::kAudioObjectSystemObject as u32,
            NonNull::from_ref(constants::default_device_address(device_type)),
            0,
            std::ptr::null(),
            NonNull::from_ref(&size),
            NonNull::new_unchecked(&mut device_id as *mut _ as *mut _),
        );
    }
    device_id
}

fn get_device_volume(device_id: AudioObjectID, device_type: DeviceType) -> i32 {
    let mut volume: f32 = 0.0;
    let size = size_of::<f32>() as u32;

    unsafe {
        let status = AudioObjectGetPropertyData(
            device_id,
            NonNull::from_ref(constants::volume_address(device_type)),
            0,
            std::ptr::null(),
            NonNull::from_ref(&size),
            NonNull::new_unchecked(&mut volume as *mut _ as *mut _),
        );
        if status != 0 {
            return 0;
        }
    }
    (volume * 100.0).round() as i32
}

fn get_device_mute(device_id: AudioObjectID, device_type: DeviceType) -> bool {
    let mut mute: u32 = 0;
    let size = size_of::<u32>() as u32;

    unsafe {
        let status = AudioObjectGetPropertyData(
            device_id,
            NonNull::from_ref(constants::mute_address(device_type)),
            0,
            std::ptr::null(),
            NonNull::from_ref(&size),
            NonNull::new_unchecked(&mut mute as *mut _ as *mut _),
        );
        if status != 0 {
            return false;
        }
    }
    mute != 0
}

fn get_device_name(device_id: AudioObjectID, device_type: DeviceType) -> String {
    const NAME_SIZE: usize = 512;
    let mut name = [0u8; NAME_SIZE];
    let size = NAME_SIZE as u32;

    unsafe {
        let status = AudioObjectGetPropertyData(
            device_id,
            NonNull::from_ref(constants::name_address(device_type)),
            0,
            std::ptr::null(),
            NonNull::from_ref(&size),
            NonNull::new_unchecked(&mut name as *mut _ as *mut _),
        );
        if status != 0 {
            return "Unknown".to_string();
        }
    }

    CStr::from_bytes_until_nul(&name)
        .unwrap_or(CStr::from_bytes_with_nul(b"Unknown\0").unwrap())
        .to_string_lossy()
        .into_owned()
}
