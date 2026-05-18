use serde::Serialize;
use std::collections::BTreeMap;

use crate::logging;
use crate::rust_api::OmniLedApi;

#[derive(Clone)]
pub struct Plugin {
    api: OmniLedApi,
    name: String,
}

#[derive(Serialize)]
struct ValueWrapper<'a, S> {
    #[serde(flatten)]
    payload: BTreeMap<&'a str, &'a S>,
}

impl Plugin {
    pub fn new(api: OmniLedApi, name: String, crate_name: &'static str) -> Self {
        logging::init(api, crate_name);

        Self { api, name }
    }

    pub fn update_with_name<S: Serialize>(
        &self,
        name: &str,
        value: &S,
    ) -> Result<(), ciborium::ser::Error<std::io::Error>> {
        let wrapper = ValueWrapper {
            payload: BTreeMap::from([(name, value)]),
        };

        let mut buffer = Vec::new();
        ciborium::into_writer(&wrapper, &mut buffer)?;
        self.api.event(&buffer);

        Ok(())
    }

    pub fn update<S: Serialize>(
        &self,
        value: &S,
    ) -> Result<(), ciborium::ser::Error<std::io::Error>> {
        self.update_with_name(&self.name, value)
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
