use objc2_core_audio::AudioObjectPropertyAddress;

use crate::DeviceType;

pub const DEFAULT_INPUT_DEVICE_SELECTOR: u32 =
    objc2_core_audio::kAudioHardwarePropertyDefaultInputDevice;
pub const DEFAULT_OUTPUT_DEVICE_SELECTOR: u32 =
    objc2_core_audio::kAudioHardwarePropertyDefaultOutputDevice;
pub const MUTE_SELECTOR: u32 = objc2_core_audio::kAudioDevicePropertyMute;
pub const NAME_SELECTOR: u32 = objc2_core_audio::kAudioDevicePropertyDeviceName;
pub const VOLUME_SELECTOR: u32 = objc2_core_audio::kAudioDevicePropertyVolumeScalar;

pub const INPUT_SCOPE: u32 = objc2_core_audio::kAudioObjectPropertyScopeInput;
pub const OUTPUT_SCOPE: u32 = objc2_core_audio::kAudioObjectPropertyScopeOutput;

macro_rules! define_get_address {
    ($name:ident, $address:ident) => {
        pub fn $name(device_type: DeviceType) -> &'static AudioObjectPropertyAddress {
            match device_type {
                DeviceType::Input => &input::$address,
                DeviceType::Output => &output::$address,
            }
        }
    };
}

define_get_address!(default_device_address, DEFAULT_DEVICE);
define_get_address!(mute_address, MUTE);
define_get_address!(name_address, NAME);
define_get_address!(volume_scalar_address, VOLUME_SCALAR);

mod input {
    use super::*;

    pub const DEFAULT_DEVICE: AudioObjectPropertyAddress = AudioObjectPropertyAddress {
        mSelector: DEFAULT_INPUT_DEVICE_SELECTOR,
        mScope: objc2_core_audio::kAudioObjectPropertyScopeGlobal,
        mElement: objc2_core_audio::kAudioObjectPropertyElementMain,
    };

    pub const MUTE: AudioObjectPropertyAddress = AudioObjectPropertyAddress {
        mSelector: MUTE_SELECTOR,
        mScope: INPUT_SCOPE,
        mElement: objc2_core_audio::kAudioObjectPropertyElementMain,
    };

    pub const NAME: AudioObjectPropertyAddress = AudioObjectPropertyAddress {
        mSelector: NAME_SELECTOR,
        mScope: INPUT_SCOPE,
        mElement: objc2_core_audio::kAudioObjectPropertyElementMain,
    };

    pub const VOLUME_SCALAR: AudioObjectPropertyAddress = AudioObjectPropertyAddress {
        mSelector: VOLUME_SELECTOR,
        mScope: INPUT_SCOPE,
        mElement: objc2_core_audio::kAudioObjectPropertyElementMain,
    };
}

mod output {
    use super::*;

    pub const DEFAULT_DEVICE: AudioObjectPropertyAddress = AudioObjectPropertyAddress {
        mSelector: DEFAULT_OUTPUT_DEVICE_SELECTOR,
        mScope: objc2_core_audio::kAudioObjectPropertyScopeGlobal,
        mElement: objc2_core_audio::kAudioObjectPropertyElementMain,
    };

    pub const MUTE: AudioObjectPropertyAddress = AudioObjectPropertyAddress {
        mSelector: MUTE_SELECTOR,
        mScope: OUTPUT_SCOPE,
        mElement: objc2_core_audio::kAudioObjectPropertyElementMain,
    };

    pub const NAME: AudioObjectPropertyAddress = AudioObjectPropertyAddress {
        mSelector: NAME_SELECTOR,
        mScope: OUTPUT_SCOPE,
        mElement: objc2_core_audio::kAudioObjectPropertyElementMain,
    };

    pub const VOLUME_SCALAR: AudioObjectPropertyAddress = AudioObjectPropertyAddress {
        mSelector: VOLUME_SELECTOR,
        mScope: OUTPUT_SCOPE,
        mElement: objc2_core_audio::kAudioObjectPropertyElementMain,
    };
}
