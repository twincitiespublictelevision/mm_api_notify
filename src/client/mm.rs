extern crate mm_client;

use mm_client::Client;

use client::client::APIClient;
use client::error::{ClientError, ClientResult};
use config::APIConfig;

pub struct MMClient {
    client: Client,
}

impl APIClient for MMClient {
    fn new(config: Option<&APIConfig>) -> ClientResult<MMClient> {
        config.ok_or(ClientError::ConfigError)
            .and_then(|conf| {
                Client::staging(conf.key.as_str(), conf.secret.as_str())
                    .or(Err(ClientError::InitializationError))
            })
            .and_then(|client| Ok(MMClient { client: client }))
    }

    fn url(&self, url: &str) -> ClientResult<String> {
        self.client
            .url(url)
            .or_else(|err| {
                error!("Failed to query {} due to {}", url, err);
                Err(err)
            })
            .map_err(ClientError::API)
    }

    fn all_shows(&self) -> ClientResult<String> {
        self.client
            .shows(vec![])
            .or_else(|err| {
                error!("Failed to query all shows due to {}", err);
                Err(err)
            })
            .map_err(ClientError::API)
    }

    fn changes(&self, since: &str) -> ClientResult<String> {
        self.client
            .changelog(vec![("since", since)])
            .or_else(|err| {
                error!("Failed to query changelog from {} due to {}", since, err);
                Err(err)
            })
            .map_err(ClientError::API)
    }
}
