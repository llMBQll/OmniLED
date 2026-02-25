mod plugin {
    tonic::include_proto!("plugin");
}

pub use plugin::*;

impl From<log::Level> for LogLevel {
    fn from(value: log::Level) -> Self {
        match value {
            log::Level::Error => LogLevel::Error,
            log::Level::Warn => LogLevel::Warn,
            log::Level::Info => LogLevel::Info,
            log::Level::Debug => LogLevel::Debug,
            log::Level::Trace => LogLevel::Trace,
        }
    }
}

impl Into<log::Level> for LogLevel {
    fn into(self) -> log::Level {
        match self {
            LogLevel::Unknown => todo!(),
            LogLevel::Error => log::Level::Error,
            LogLevel::Warn => log::Level::Warn,
            LogLevel::Info => log::Level::Info,
            LogLevel::Debug => log::Level::Debug,
            LogLevel::Trace => log::Level::Trace,
        }
    }
}

impl From<log::LevelFilter> for LogLevelFilter {
    fn from(value: log::LevelFilter) -> Self {
        match value {
            log::LevelFilter::Off => LogLevelFilter::Off,
            log::LevelFilter::Error => LogLevelFilter::Error,
            log::LevelFilter::Warn => LogLevelFilter::Warn,
            log::LevelFilter::Info => LogLevelFilter::Info,
            log::LevelFilter::Debug => LogLevelFilter::Debug,
            log::LevelFilter::Trace => LogLevelFilter::Trace,
        }
    }
}

impl Into<log::LevelFilter> for LogLevelFilter {
    fn into(self) -> log::LevelFilter {
        match self {
            LogLevelFilter::Unknown => todo!(),
            LogLevelFilter::Off => log::LevelFilter::Off,
            LogLevelFilter::Error => log::LevelFilter::Error,
            LogLevelFilter::Warn => log::LevelFilter::Warn,
            LogLevelFilter::Info => log::LevelFilter::Info,
            LogLevelFilter::Debug => log::LevelFilter::Debug,
            LogLevelFilter::Trace => log::LevelFilter::Trace,
        }
    }
}

macro_rules! cast_and_into_field {
    ($from:ty, $to:ty, $variant:expr) => {
        impl Into<Field> for $from {
            fn into(self) -> Field {
                Field {
                    field: Some($variant(self as $to)),
                }
            }
        }
    };
}

macro_rules! into_field {
    ($from:ty, $variant:expr) => {
        impl Into<Field> for $from {
            fn into(self) -> Field {
                Field {
                    field: Some($variant(self)),
                }
            }
        }
    };
}

// None
into_field!(None, field::Field::FNone);

// Boolean values
into_field!(bool, field::Field::FBool);

// Integer values
cast_and_into_field!(i8, i64, field::Field::FInteger);
cast_and_into_field!(i16, i64, field::Field::FInteger);
cast_and_into_field!(i32, i64, field::Field::FInteger);
into_field!(i64, field::Field::FInteger);
cast_and_into_field!(i128, i64, field::Field::FInteger);
cast_and_into_field!(u8, i64, field::Field::FInteger);
cast_and_into_field!(u16, i64, field::Field::FInteger);
cast_and_into_field!(u32, i64, field::Field::FInteger);
cast_and_into_field!(u64, i64, field::Field::FInteger);
cast_and_into_field!(u128, i64, field::Field::FInteger);

// Floating point values
cast_and_into_field!(f32, f64, field::Field::FFloat);
into_field!(f64, field::Field::FFloat);

// String values
into_field!(String, field::Field::FString);

impl Into<Field> for &str {
    fn into(self) -> Field {
        Field {
            field: Some(field::Field::FString(self.to_owned())),
        }
    }
}

// Array values
into_field!(Array, field::Field::FArray);

impl<T: Into<Field>> Into<Field> for Vec<T> {
    fn into(self) -> Field {
        let array = Array {
            items: self.into_iter().map(|entry| entry.into()).collect(),
        };

        array.into()
    }
}

// Image values
into_field!(ImageData, field::Field::FImageData);

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
            ImageFormat::Unknown => todo!(),
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
            ImageFormat::Pcx => return Err("Pcx image format is deprecated"),
        };
        Ok(res)
    }
}
