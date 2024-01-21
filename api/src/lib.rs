use prost::bytes::Bytes;
use prost::Message;
use std::io::Read;
use ureq::{Agent, Error, Response};

pub mod types {
    include!(concat!(env!("OUT_DIR"), "/types.rs"));
}

use types::field::Field::{FArray, FBool, FBytes, FFloat, FImage, FInteger, FString};
use types::{Array, Event, EventReply, Field, Image, Table};

#[derive(Debug)]
pub struct Api {
    agent: Agent,
    address: String,
    name: String,
}

impl Api {
    pub fn new(address: &str, application_name: &str) -> Self {
        Self {
            agent: Agent::new(),
            address: address.to_string(),
            name: application_name.to_string(),
        }
    }

    pub fn update(&self, fields: Table) {
        self.update_with_name(&self.name, fields)
    }

    pub fn update_with_name(&self, name: &str, fields: Table) {
        let event = Event {
            name: name.to_string(),
            fields: Some(fields),
        };
        let update_data = event.encode_to_vec();
        match self.call("/update", &update_data) {
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

    fn call(&self, endpoint: &str, bytes: &Vec<u8>) -> Result<Response, Error> {
        let url = format!("http://{}{}", self.address, endpoint);
        self.agent.post(&url).send_bytes(bytes)
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

// Byte values
into_field!(Vec<u8>, FBytes);

// Image values
into_field!(Image, FImage);
