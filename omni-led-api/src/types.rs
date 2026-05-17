mod plugin {
    include!(concat!(env!("OUT_DIR"), "/plugin.rs"));
}

pub use plugin::*;

macro_rules! cast_and_into_value {
    ($from:ty, $to:ty, $variant:expr) => {
        impl Into<Value> for $from {
            fn into(self) -> Value {
                Value {
                    value: Some($variant(self as $to)),
                }
            }
        }
    };
}

macro_rules! into_value {
    ($from:ty, $variant:expr) => {
        impl Into<Value> for $from {
            fn into(self) -> Value {
                Value {
                    value: Some($variant(self)),
                }
            }
        }
    };
}

// Boolean values
into_value!(bool, value::Value::VBool);

// Integer values
cast_and_into_value!(i8, i64, value::Value::VInteger);
cast_and_into_value!(i16, i64, value::Value::VInteger);
cast_and_into_value!(i32, i64, value::Value::VInteger);
into_value!(i64, value::Value::VInteger);
cast_and_into_value!(i128, i64, value::Value::VInteger);
cast_and_into_value!(u8, i64, value::Value::VInteger);
cast_and_into_value!(u16, i64, value::Value::VInteger);
cast_and_into_value!(u32, i64, value::Value::VInteger);
cast_and_into_value!(u64, i64, value::Value::VInteger);
cast_and_into_value!(u128, i64, value::Value::VInteger);

// Floating point values
cast_and_into_value!(f32, f64, value::Value::VFloat);
into_value!(f64, value::Value::VFloat);

// String values
into_value!(String, value::Value::VString);

impl Into<Value> for &str {
    fn into(self) -> Value {
        Value {
            value: Some(value::Value::VString(self.to_owned())),
        }
    }
}

impl Into<Value> for char {
    fn into(self) -> Value {
        Value {
            value: Some(value::Value::VString(self.to_string())),
        }
    }
}

// Array values
into_value!(Array, value::Value::VArray);

impl<T: Into<Value>> Into<Value> for Vec<T> {
    fn into(self) -> Value {
        let array = Array {
            items: self.into_iter().map(|entry| entry.into()).collect(),
        };

        array.into()
    }
}

// Table values
into_value!(Table, value::Value::VTable);

// Image values
into_value!(ImageData, value::Value::VImageData);

impl TryFrom<image::ImageFormat> for ImageFormat {
    type Error = &'static str;

    fn try_from(value: image::ImageFormat) -> Result<Self, Self::Error> {
        let res = match value {
            image::ImageFormat::Png => ImageFormat::Png,
            image::ImageFormat::Jpeg => ImageFormat::Jpeg,
            image::ImageFormat::Gif => ImageFormat::Gif,
            image::ImageFormat::WebP => ImageFormat::Webp,
            image::ImageFormat::Pnm => ImageFormat::Pnm,
            image::ImageFormat::Tiff => ImageFormat::Tiff,
            image::ImageFormat::Tga => ImageFormat::Tga,
            image::ImageFormat::Dds => ImageFormat::Dds,
            image::ImageFormat::Bmp => ImageFormat::Bmp,
            image::ImageFormat::Ico => ImageFormat::Ico,
            image::ImageFormat::Hdr => ImageFormat::Hdr,
            image::ImageFormat::OpenExr => ImageFormat::OpenExr,
            image::ImageFormat::Farbfeld => ImageFormat::Farbfeld,
            image::ImageFormat::Avif => ImageFormat::Avif,
            image::ImageFormat::Qoi => ImageFormat::Qoi,
            #[allow(deprecated)]
            image::ImageFormat::Pcx => return Err("Pcx image format is deprecated"),
            _ => return Err("Unknown image format"),
        };
        Ok(res)
    }
}

impl TryInto<image::ImageFormat> for ImageFormat {
    type Error = &'static str;

    fn try_into(self) -> Result<image::ImageFormat, Self::Error> {
        let res = match self {
            ImageFormat::Unspecified => todo!(),
            ImageFormat::Png => image::ImageFormat::Png,
            ImageFormat::Jpeg => image::ImageFormat::Jpeg,
            ImageFormat::Gif => image::ImageFormat::Gif,
            ImageFormat::Webp => image::ImageFormat::WebP,
            ImageFormat::Pnm => image::ImageFormat::Pnm,
            ImageFormat::Tiff => image::ImageFormat::Tiff,
            ImageFormat::Tga => image::ImageFormat::Tga,
            ImageFormat::Dds => image::ImageFormat::Dds,
            ImageFormat::Bmp => image::ImageFormat::Bmp,
            ImageFormat::Ico => image::ImageFormat::Ico,
            ImageFormat::Hdr => image::ImageFormat::Hdr,
            ImageFormat::OpenExr => image::ImageFormat::OpenExr,
            ImageFormat::Farbfeld => image::ImageFormat::Farbfeld,
            ImageFormat::Avif => image::ImageFormat::Avif,
            ImageFormat::Qoi => image::ImageFormat::Qoi,
            #[allow(deprecated)]
            ImageFormat::Pcx => return Err("Pcx image format is deprecated"),
        };
        Ok(res)
    }
}
