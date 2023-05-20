use ureq::Agent;
use crate::screen::screen::{Screen, Size};
use crate::screen::screen::Result;

pub struct SteelseriesEngine {
    size: Size,
    agent: Agent,
    address: Option<String>,
}

const GAME: &str = "RUST_STEELSERIES_OLED";
const GAME_DISPLAY_NAME: &str = "[Rust] Steelseries OLED";
const DEVELOPER: &str = "MBQ";
const TIMEOUT: u32 = 60000;

impl SteelseriesEngine {
    pub fn new() -> Self {
        Self {
            size: Size { width: 128, height: 40 },
            agent: Agent::new(),
            address: None,
        }
    }
}

impl Screen for SteelseriesEngine {
    fn init(&mut self) -> Result<()> {
        let metadata = serde_json::json!({
            "game": GAME,
            "game_display_name": GAME_DISPLAY_NAME,
            "developer": DEVELOPER,
            "deinitialize_timer_length_ms": TIMEOUT
        });
        let metadata = serde_json::to_string(&metadata).unwrap();
        // todo!("self.game_metadata(&metadata)");

        let handlers = serde_json::json!({
            "game": GAME,
            "golisp": "(handler \"UPDATE\" (lambda (data) (on-device 'screened show-image: (list-to-bytearray (image-data: (frame: data))))))"
        });
        let handlers = serde_json::to_string(&handers).unwrap();
        // todo!("self.load_golisp_handlers(&handlers)");
    }

    fn size(&mut self) -> Result<Size> {
        Ok(self.size)
    }

    fn update(&mut self, pixels: &Vec<u8>) -> Result<()> {
        todo!()
    }

    fn name(&self) -> Result<String> {
        Ok(String::from("Steelseries Engine"))
    }
}