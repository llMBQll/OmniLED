use std::sync::Arc;
use tokio::{runtime::Handle, sync::Mutex};

use crate::{
    logging,
    types::{EventData, LogData, LogLevel, Table, plugin_client::PluginClient},
};

#[derive(Clone)]
pub struct Plugin {
    inner: Arc<Mutex<PluginInner>>,
}

struct PluginInner {
    name: String,
    client: PluginClient<tonic::transport::Channel>,
}

impl Plugin {
    pub async fn new(
        name: &str,
        crate_name: &'static str,
        url: &str,
    ) -> Result<Self, tonic::transport::Error> {
        let client = PluginClient::connect(format!("http://{url}")).await?;

        let plugin = Self {
            inner: Arc::new(Mutex::new(PluginInner {
                name: name.to_string(),
                client: client,
            })),
        };

        logging::init(Handle::current(), plugin.clone(), crate_name);

        Ok(plugin)
    }

    pub async fn log(
        &self,
        log_level: LogLevel,
        location: String,
        message: String,
    ) -> Result<(), tonic::Status> {
        self.inner
            .lock()
            .await
            .client
            .log(LogData {
                log_level: log_level as i32,
                location: location,
                message: message,
            })
            .await?;
        Ok(())
    }

    pub async fn update_with_name(&self, name: &str, fields: Table) -> Result<(), tonic::Status> {
        let mut inner = self.inner.lock().await;
        Self::update_impl(&mut inner.client, name.to_string(), fields).await
    }

    pub async fn update(&self, fields: Table) -> Result<(), tonic::Status> {
        let mut inner = self.inner.lock().await;
        let name = inner.name.clone();
        Self::update_impl(&mut inner.client, name, fields).await
    }

    async fn update_impl(
        client: &mut PluginClient<tonic::transport::Channel>,
        name: String,
        fields: Table,
    ) -> Result<(), tonic::Status> {
        client
            .event(EventData {
                name,
                fields: Some(fields),
            })
            .await?;
        Ok(())
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
