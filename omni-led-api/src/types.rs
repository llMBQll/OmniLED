use ciborium::tag::Required;
use image::ImageFormat;
use serde::{Deserialize, Serialize, Serializer};

use crate::c_api;

pub trait Tagged {
    const TAG: u64;
}

// Deserialize can derived as when handling, we get the tag number and parsed untagged value
// Only thing remainging is the dynamic `ciborium::Value` to static `Image`
#[derive(Deserialize)]
pub struct Image {
    pub format: ImageFormat,
    pub bytes: Vec<u8>,
}

impl Tagged for Image {
    const TAG: u64 = c_api::MBQ_OMNI_LED_TAG_IMAGE as u64;
}

impl Serialize for Image {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct ImageFields<'a> {
            format: &'a ImageFormat,
            bytes: &'a Vec<u8>,
        }

        Required::<ImageFields, { Image::TAG }>(ImageFields {
            format: &self.format,
            bytes: &self.bytes,
        })
        .serialize(serializer)
    }
}
