use prost::bytes::Bytes;
use prost::Message;
use std::io::Read;
use ureq::{Agent, Error};

pub mod types {
    include!(concat!(env!("OUT_DIR"), "/types.rs"));
}

use types::field::Field::{FArray, FBool, FFloat, FImage, FInteger, FString};
use types::{Array, Event, EventReply, Field, Image, LogLevel, LogMessage, Table};

#[derive(Debug)]
pub struct Api {
    agent: Agent,
    address: String,
    name: String,
}

impl Api {
    pub fn new(address: &str, application_name: &str) -> Self {
        let api = Self {
            agent: Agent::new(),
            address: address.to_string(),
            name: application_name.to_string(),
        };
        println!("{} connected", api.name);
        api.log("Connected", LogLevel::Debug);
        api
    }

    pub fn update(&self, fields: Table) {
        self.update_with_name(&self.name, fields)
    }

    pub fn update_with_name(&self, name: &str, fields: Table) {
        let event = Event {
            name: name.to_string(),
            fields: Some(fields),
        };
        let bytes = event.encode_to_vec();
        self.call("/update", &bytes);
    }

    pub fn log(&self, message: &str, level: LogLevel) {
        let log_message = LogMessage {
            name: self.name.clone(),
            message: message.to_owned(),
            severity: level.into(),
        };
        let bytes = log_message.encode_to_vec();
        self.call("/log", &bytes);
    }

    fn call(&self, endpoint: &str, bytes: &Vec<u8>) {
        let url = format!("http://{}{}", self.address, endpoint);
        match self.agent.post(&url).send_bytes(bytes) {
            Ok(_) => {}
            Err(err) => match err {
                Error::Status(status, response) => {
                    let mut bytes = Vec::new();
                    response.into_reader().read_to_end(&mut bytes).unwrap();
                    let bytes = Bytes::from(bytes);
                    let reply = EventReply::decode(bytes).unwrap_or(EventReply { error: None });
                    println!(
                        "[{}] [{status}] {}",
                        self.name,
                        reply.error.unwrap_or("Unknown error".to_string())
                    );
                }
                Error::Transport(transport) => println!("[{}] {transport}", self.name),
            },
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

// Boolean values
into_field!(bool, FBool);

// Integer values
cast_and_into_field!(i8, i64, FInteger);
cast_and_into_field!(i16, i64, FInteger);
cast_and_into_field!(i32, i64, FInteger);
into_field!(i64, FInteger);
cast_and_into_field!(i128, i64, FInteger);
cast_and_into_field!(u8, i64, FInteger);
cast_and_into_field!(u16, i64, FInteger);
cast_and_into_field!(u32, i64, FInteger);
cast_and_into_field!(u64, i64, FInteger);
cast_and_into_field!(u128, i64, FInteger);

// Floating point values
cast_and_into_field!(f32, f64, FFloat);
into_field!(f64, FFloat);

// String values
into_field!(String, FString);

impl Into<Field> for &str {
    fn into(self) -> Field {
        Field {
            field: Some(FString(self.to_owned())),
        }
    }
}

// Array values
into_field!(Array, FArray);

impl<T: Into<Field>> Into<Field> for Vec<T> {
    fn into(self) -> Field {
        let array = Array {
            items: self.into_iter().map(|entry| entry.into()).collect(),
        };

        array.into()
    }
}

// Image values
into_field!(Image, FImage);
