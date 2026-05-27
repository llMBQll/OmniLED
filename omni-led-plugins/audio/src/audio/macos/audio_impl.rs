use objc2_core_audio::{
    AudioObjectAddPropertyListener, AudioObjectGetPropertyData, AudioObjectID,
    AudioObjectPropertyAddress, AudioObjectRemovePropertyListener,
};
use std::ffi::{CStr, c_void};
use std::mem::size_of;
use std::ptr::NonNull;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use crate::audio::macos::addresses;
use crate::{DeviceData, DeviceType};

pub struct AudioContext {
    pub tx: Sender<(DeviceData, DeviceType)>,
    pub current_ids: [AudioObjectID; 2],
}

pub struct AudioImpl {
    ctx: *mut c_void,
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

        let context = Arc::new(Mutex::new(AudioContext {
            tx: tx.clone(),
            current_ids,
        }));

        let ctx = Arc::into_raw(context) as *mut c_void;

        unsafe {
            register_device_listeners(
                ctx,
                current_ids[DeviceType::Input as usize],
                DeviceType::Input,
            );
            register_device_listeners(
                ctx,
                current_ids[DeviceType::Output as usize],
                DeviceType::Output,
            );

            _ = AudioObjectAddPropertyListener(
                objc2_core_audio::kAudioObjectSystemObject as u32,
                NonNull::from_ref(addresses::default_device_address(DeviceType::Input)),
                Some(system_listener),
                ctx,
            );
            _ = AudioObjectAddPropertyListener(
                objc2_core_audio::kAudioObjectSystemObject as u32,
                NonNull::from_ref(addresses::default_device_address(DeviceType::Output)),
                Some(system_listener),
                ctx,
            );
        }

        send_device_update(
            current_ids[DeviceType::Input as usize],
            &tx,
            DeviceType::Input,
        );
        send_device_update(
            current_ids[DeviceType::Output as usize],
            &tx,
            DeviceType::Output,
        );

        Self { ctx }
    }
}

impl Drop for AudioImpl {
    fn drop(&mut self) {
        unsafe {
            let context = Arc::from_raw(self.ctx as *const Mutex<AudioContext>);
            let ctx = context.lock().unwrap();

            _ = AudioObjectRemovePropertyListener(
                objc2_core_audio::kAudioObjectSystemObject as u32,
                NonNull::from_ref(addresses::default_device_address(DeviceType::Input)),
                Some(system_listener),
                self.ctx,
            );
            _ = AudioObjectRemovePropertyListener(
                objc2_core_audio::kAudioObjectSystemObject as u32,
                NonNull::from_ref(addresses::default_device_address(DeviceType::Output)),
                Some(system_listener),
                self.ctx,
            );

            unregister_device_listeners(
                self.ctx,
                ctx.current_ids[DeviceType::Input as usize],
                DeviceType::Input,
            );
            unregister_device_listeners(
                self.ctx,
                ctx.current_ids[DeviceType::Output as usize],
                DeviceType::Output,
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
                objc2_core_audio::kAudioHardwarePropertyDefaultInputDevice => DeviceType::Input,
                objc2_core_audio::kAudioHardwarePropertyDefaultOutputDevice => DeviceType::Output,
                _ => continue,
            };

            let new_device_id = get_default_device(device_type);
            let idx = device_type as usize;

            if new_device_id != ctx.current_ids[idx] && new_device_id != 0 {
                unregister_device_listeners(in_client_data, ctx.current_ids[idx], device_type);

                register_device_listeners(in_client_data, new_device_id, device_type);
                ctx.current_ids[idx] = new_device_id;

                send_device_update(new_device_id, &ctx.tx, device_type);
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
                objc2_core_audio::kAudioObjectPropertyScopeInput => DeviceType::Input,
                objc2_core_audio::kAudioObjectPropertyScopeOutput => DeviceType::Output,
                _ => continue,
            };

            if in_object_id == ctx.current_ids[device_type as usize]
                && (addr.mSelector == objc2_core_audio::kAudioDevicePropertyVolumeScalar
                    || addr.mSelector == objc2_core_audio::kAudioDevicePropertyMute)
            {
                update_input = device_type == DeviceType::Input;
                update_output = device_type == DeviceType::Output;
            }
        }

        if update_input {
            send_device_update(in_object_id, &ctx.tx, DeviceType::Input);
        }
        if update_output {
            send_device_update(in_object_id, &ctx.tx, DeviceType::Output);
        }
    }
    0
}

fn register_device_listeners(ctx: *mut c_void, device_id: AudioObjectID, device_type: DeviceType) {
    let _ = unsafe {
        AudioObjectAddPropertyListener(
            device_id,
            NonNull::from_ref(addresses::mute_address(device_type)),
            Some(device_listener),
            ctx,
        )
    };
    let _ = unsafe {
        AudioObjectAddPropertyListener(
            device_id,
            NonNull::from_ref(addresses::volume_scalar_address(device_type)),
            Some(device_listener),
            ctx,
        )
    };
}

fn unregister_device_listeners(
    ctx: *mut c_void,
    device_id: AudioObjectID,
    device_type: DeviceType,
) {
    let _ = unsafe {
        AudioObjectRemovePropertyListener(
            device_id,
            NonNull::from_ref(addresses::mute_address(device_type)),
            Some(device_listener),
            ctx,
        )
    };
    let _ = unsafe {
        AudioObjectRemovePropertyListener(
            device_id,
            NonNull::from_ref(addresses::volume_scalar_address(device_type)),
            Some(device_listener),
            ctx,
        )
    };
}

fn send_device_update(
    device_id: AudioObjectID,
    tx: &Sender<(DeviceData, DeviceType)>,
    device_type: DeviceType,
) {
    let volume = get_device_volume(device_id, device_type);
    let mute = get_device_mute(device_id, device_type);
    let name = get_device_name(device_id, device_type);

    let _ = tx.send((DeviceData::new(true, mute, volume, Some(name)), device_type));
}

fn get_default_device(device_type: DeviceType) -> AudioObjectID {
    let mut device_id: AudioObjectID = 0;
    let size = size_of::<AudioObjectID>() as u32;

    unsafe {
        _ = AudioObjectGetPropertyData(
            objc2_core_audio::kAudioObjectSystemObject as u32,
            NonNull::from_ref(addresses::default_device_address(device_type)),
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
            NonNull::from_ref(addresses::volume_scalar_address(device_type)),
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
            NonNull::from_ref(addresses::mute_address(device_type)),
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
            NonNull::from_ref(addresses::name_address(device_type)),
            0,
            std::ptr::null(),
            NonNull::from_ref(&size),
            NonNull::new_unchecked(&mut name as *mut _ as *mut _),
        );
        if status != 0 {
            return "Unknown Device".to_string();
        }
    }

    CStr::from_bytes_until_nul(&name)
        .unwrap_or(CStr::from_bytes_with_nul(b"Unknown\0").unwrap())
        .to_string_lossy()
        .into_owned()
}
