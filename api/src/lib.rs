use prost::bytes::Bytes;
use prost::Message;
use serde::{Deserialize, Serialize};
use ureq::{Agent, Error, Response};

pub mod types {
    include!(concat!(env!("OUT_DIR"), "/types.rs"));
}

use types::field::Field::{FArray, FBool, FBytes, FFloat, FImage, FInteger, FString, FTable};
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
                    let response: Bytes = response.into();
                    let reply = EventReply::decode(response).unwrap_or(EventReply { error: None });
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

#[derive(Serialize, Deserialize)]
struct Reply {
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Serialize)]
struct UpdateData<'a, 'b, T: Serialize> {
    name: &'a str,
    fields: &'b T,
}

impl Into<Field> for bool {
    fn into(self) -> Field {
        Field {
            field: Some(FBool(self)),
        }
    }
}

impl Into<Field> for u32 {
    fn into(self) -> Field {
        Field {
            field: Some(FInteger(self as i64)),
        }
    }
}

impl Into<Field> for i32 {
    fn into(self) -> Field {
        Field {
            field: Some(FInteger(self as i64)),
        }
    }
}

impl Into<Field> for i64 {
    fn into(self) -> Field {
        Field {
            field: Some(FInteger(self)),
        }
    }
}

impl Into<Field> for f64 {
    fn into(self) -> Field {
        Field {
            field: Some(FFloat(self)),
        }
    }
}

impl Into<Field> for &str {
    fn into(self) -> Field {
        Field {
            field: Some(FString(self.to_owned())),
        }
    }
}

impl Into<Field> for String {
    fn into(self) -> Field {
        Field {
            field: Some(FString(self)),
        }
    }
}

impl Into<Field> for Array {
    fn into(self) -> Field {
        Field {
            field: Some(FArray(self)),
        }
    }
}

impl Into<Field> for Table {
    fn into(self) -> Field {
        Field {
            field: Some(FTable(self)),
        }
    }
}

impl Into<Field> for Vec<u8> {
    fn into(self) -> Field {
        Field {
            field: Some(FBytes(self)),
        }
    }
}

impl Into<Field> for Image {
    fn into(self) -> Field {
        Field {
            field: Some(FImage(self)),
        }
    }
}
