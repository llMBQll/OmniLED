mod plugin {
    tonic::include_proto!("plugin");
}

pub use plugin::*;

use prost::UnknownEnumValue;

#[derive(Debug)]
pub struct Plugin {
    name: String,
    client: plugin_client::PluginClient<tonic::transport::Channel>,
}

impl Plugin {
    pub async fn new(name: &str, url: &str) -> Result<Self, tonic::transport::Error> {
        Ok(Self {
            name: name.to_string(),
            client: plugin_client::PluginClient::connect(format!("http://{url}")).await?,
        })
    }

    pub async fn update_with_name(
        &mut self,
        name: &str,
        fields: Table,
    ) -> Result<tonic::Response<EventResponse>, tonic::Status> {
        let data = EventData {
            name: name.to_string(),
            fields: Some(fields),
        };

        self.client.event(data).await
    }

    pub async fn update(
        &mut self,
        fields: Table,
    ) -> Result<tonic::Response<EventResponse>, tonic::Status> {
        let name = self.name.clone();
        self.update_with_name(&name, fields).await
    }

    pub async fn log(
        &mut self,
        message: &str,
        level: LogLevel,
    ) -> Result<tonic::Response<LogResponse>, tonic::Status> {
        let message = LogMessage {
            name: self.name.clone(),
            message: message.to_string(),
            severity: level.into(),
        };
        self.client.log(message).await
    }

    pub fn log_level_from_integer(value: i32) -> Result<log::Level, UnknownEnumValue> {
        match LogLevel::try_from(value) {
            Ok(LogLevel::Error) => Ok(log::Level::Error),
            Ok(LogLevel::Warn) => Ok(log::Level::Warn),
            Ok(LogLevel::Info) => Ok(log::Level::Info),
            Ok(LogLevel::Debug) => Ok(log::Level::Debug),
            Ok(LogLevel::Trace) => Ok(log::Level::Trace),
            Err(err) => Err(err),
        }
    }

    pub fn is_valid_identifier(identifier: &str) -> bool {
        if identifier.len() == 0 {
            return false;
        }

        let mut chars = identifier.chars();

        let first = chars.next().unwrap();
        if first != '_' && (first < 'A' || first > 'Z') {
            return false;
        }

        for c in chars {
            if c != '_' && (c < 'A' || c > 'Z') && (c < '0' || c > '9') {
                return false;
            }
        }

        true
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
into_field!(Image, field::Field::FImage);
