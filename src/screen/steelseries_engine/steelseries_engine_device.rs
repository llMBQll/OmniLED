use crate::screen::screen::{Screen, Size};
use crate::screen::screen::Result;
use crate::screen::steelseries_engine::api;
use crate::screen::supported_devices::device_info::SteelseriesEngineDeviceSettings;

pub struct SteelseriesEngineDevice {
    name: String,
    size: Size,
}

impl SteelseriesEngineDevice {
    pub fn new(settings: SteelseriesEngineDeviceSettings) -> Self {
        Self {
            name: settings.name,
            size: settings.screen_size,
        }
    }
}

impl Screen for SteelseriesEngineDevice {
    fn init(&mut self) -> Result<()> {
        Ok(())
    }

    fn size(&mut self) -> Result<Size> {
        Ok(self.size)
    }

    fn update(&mut self, pixels: &Vec<u8>) -> Result<()> {
        api::update(pixels);
        Ok(())
    }

    fn name(&self) -> Result<String> {
        Ok(self.name.clone())
    }
}