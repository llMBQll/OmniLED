use objc2_core_audio::AudioObjectPropertyAddress;

use crate::DeviceType;

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
    use objc2_core_audio::AudioObjectPropertyAddress;

    pub const DEFAULT_DEVICE: AudioObjectPropertyAddress = AudioObjectPropertyAddress {
        mSelector: objc2_core_audio::kAudioHardwarePropertyDefaultInputDevice,
        mScope: objc2_core_audio::kAudioObjectPropertyScopeGlobal,
        mElement: objc2_core_audio::kAudioObjectPropertyElementMain,
    };

    pub const MUTE: AudioObjectPropertyAddress = AudioObjectPropertyAddress {
        mSelector: objc2_core_audio::kAudioDevicePropertyMute,
        mScope: objc2_core_audio::kAudioObjectPropertyScopeInput,
        mElement: objc2_core_audio::kAudioObjectPropertyElementMain,
    };

    pub const NAME: AudioObjectPropertyAddress = AudioObjectPropertyAddress {
        mSelector: objc2_core_audio::kAudioDevicePropertyDeviceName,
        mScope: objc2_core_audio::kAudioObjectPropertyScopeInput,
        mElement: objc2_core_audio::kAudioObjectPropertyElementMain,
    };

    pub const VOLUME_SCALAR: AudioObjectPropertyAddress = AudioObjectPropertyAddress {
        mSelector: objc2_core_audio::kAudioDevicePropertyVolumeScalar,
        mScope: objc2_core_audio::kAudioObjectPropertyScopeInput,
        mElement: objc2_core_audio::kAudioObjectPropertyElementMain,
    };
}

mod output {
    use objc2_core_audio::AudioObjectPropertyAddress;

    pub const DEFAULT_DEVICE: AudioObjectPropertyAddress = AudioObjectPropertyAddress {
        mSelector: objc2_core_audio::kAudioHardwarePropertyDefaultOutputDevice,
        mScope: objc2_core_audio::kAudioObjectPropertyScopeGlobal,
        mElement: objc2_core_audio::kAudioObjectPropertyElementMain,
    };

    pub const MUTE: AudioObjectPropertyAddress = AudioObjectPropertyAddress {
        mSelector: objc2_core_audio::kAudioDevicePropertyMute,
        mScope: objc2_core_audio::kAudioObjectPropertyScopeOutput,
        mElement: objc2_core_audio::kAudioObjectPropertyElementMain,
    };

    pub const NAME: AudioObjectPropertyAddress = AudioObjectPropertyAddress {
        mSelector: objc2_core_audio::kAudioDevicePropertyDeviceName,
        mScope: objc2_core_audio::kAudioObjectPropertyScopeOutput,
        mElement: objc2_core_audio::kAudioObjectPropertyElementMain,
    };

    pub const VOLUME_SCALAR: AudioObjectPropertyAddress = AudioObjectPropertyAddress {
        mSelector: objc2_core_audio::kAudioDevicePropertyVolumeScalar,
        mScope: objc2_core_audio::kAudioObjectPropertyScopeOutput,
        mElement: objc2_core_audio::kAudioObjectPropertyElementMain,
    };
}
