use prost::{EncodeError, Message};

use crate::logging;
use crate::rust_api::OmniLedApi;
use crate::types::Table;

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

    pub fn update_with_name(&self, name: &str, values: Table) -> Result<(), EncodeError> {
        let mut event_data = Table::default();
        event_data.items.insert(name.into(), values.into());

        let mut message = Vec::new();
        event_data.encode(&mut message)?;
        self.api.event(&message);

        Ok(())
    }

    pub fn update(&self, values: Table) -> Result<(), EncodeError> {
        self.update_with_name(&self.name, values)
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
    ($api:ident) => {{
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
        omni_led_api::plugin::Plugin::new($api, plugin_name, crate_name)
    }};
}
