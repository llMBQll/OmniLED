use prost::{EncodeError, Message};

use crate::logging;
use crate::rust_api::OmniLedApi;
use crate::types::{EventData, Table};

#[derive(Clone)]
pub struct Plugin {
    api: OmniLedApi,
    name: String,
}

impl Plugin {
    pub fn new(api: OmniLedApi, name: String, crate_name: &'static str) -> Self {
        logging::init(api, crate_name);

        Self { api, name }
    }

    pub fn update_with_name(&self, name: &str, fields: Table) -> Result<(), EncodeError> {
        let event_data = EventData {
            name: name.to_string(),
            fields: Some(fields),
        };

        let mut message = Vec::new();
        event_data.encode(&mut message)?;

        self.api.event(&message);

        Ok(())
    }

    pub fn update(&self, fields: Table) -> Result<(), EncodeError> {
        self.update_with_name(&self.name, fields)
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

#[macro_export]
macro_rules! new_plugin {
    ($c_api:ident) => {{
        let api = omni_led_new_api::rust_api::OmniLedApi::new($c_api);
        let crate_name = env!("CARGO_PKG_NAME");
        let plugin_name: String = crate_name
            .chars()
            .map(|c| {
                if c == '-' {
                    '_'
                } else {
                    c.to_ascii_uppercase()
                }
            })
            .collect();
        omni_led_new_api::plugin::Plugin::new(api, plugin_name, crate_name)
    }};
}
