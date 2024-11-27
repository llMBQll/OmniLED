use crate::logging;
use crate::types::{plugin_client, EventData, Table};
use tokio::runtime::Handle;
use tokio::sync::mpsc::channel;
use tokio_stream::wrappers::ReceiverStream;

#[derive(Debug)]
pub struct Plugin {
    name: String,
    client: plugin_client::PluginClient<tonic::transport::Channel>,
}

impl Plugin {
    pub async fn new(name: &str, url: &str) -> Result<Self, tonic::transport::Error> {
        let mut client = plugin_client::PluginClient::connect(format!("http://{url}")).await?;

        let (tx, rx) = channel(128);
        let stream = ReceiverStream::new(rx);

        let log_level: log::LevelFilter = match client.log(stream).await {
            Ok(response) => response.into_inner().log_level_filter().into(),
            Err(_) => todo!(),
        };
        logging::init(Handle::current(), tx, log_level);

        Ok(Self {
            name: name.to_string(),
            client,
        })
    }

    pub async fn update_with_name(
        &mut self,
        name: &str,
        fields: Table,
    ) -> Result<(), tonic::Status> {
        let data = EventData {
            name: name.to_string(),
            fields: Some(fields),
        };

        self.client.event(data).await?;
        Ok(())
    }

    pub async fn update(&mut self, fields: Table) -> Result<(), tonic::Status> {
        let name = self.name.clone();

        self.update_with_name(&name, fields).await?;
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
