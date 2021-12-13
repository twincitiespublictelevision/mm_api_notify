extern crate mm_client;

use mm_client::Client;
use mm_client::MMCError;

use client::client::APIClient;
use client::error::{ClientError, ClientResult};
use config::APIConfig;

pub struct MMClient {
    client: Client,
}

impl APIClient for MMClient {
    fn new(config: Option<&APIConfig>) -> ClientResult<MMClient> {
        config
            .ok_or(ClientError::ConfigError)
            .and_then(|conf| {
                Client::new(conf.key.as_str(), conf.secret.as_str())
                    .or(Err(ClientError::InitializationError))
            })
            .and_then(|client| Ok(MMClient { client: client }))
    }

    fn url(&self, url: &str) -> ClientResult<String> {
        let mut lookup_url = String::new();
        lookup_url.push_str(url);
        lookup_url.push(if url.contains('?') { '&' } else { '?' });
        lookup_url.push_str("platform-slug");
        lookup_url.push_str("=");
        lookup_url.push_str("partnerplayer");

        self.client
            .url(lookup_url.as_str())
            .or_else(|err| {
                match err {
                    MMCError::ResourceNotFound => {}
                    MMCError::NotAuthorized => {}
                    _ => {
                        error!("Failed to query {} due to {}", url, err);
                    }
                };

                Err(err)
            })
            .map_err(ClientError::API)
    }

    fn show(&self, id: &str) -> ClientResult<String> {
        self.client
            .show(id, Some(vec![("platform-slug", "partnerplayer")]))
            .or_else(|err| {
                match err {
                    MMCError::ResourceNotFound => {}
                    MMCError::NotAuthorized => {}
                    _ => {
                        error!("Failed to query show {} due to {}", id, err);
                    }
                };

                Err(err)
            })
            .map_err(ClientError::API)
    }

    fn franchise(&self, id: &str) -> ClientResult<String> {
        self.client
            .franchise(id, Some(vec![("platform-slug", "partnerplayer")]))
            .or_else(|err| {
                match err {
                    MMCError::ResourceNotFound => {}
                    MMCError::NotAuthorized => {}
                    _ => {
                        error!("Failed to query franchise {} due to {}", id, err);
                    }
                };

                Err(err)
            })
            .map_err(ClientError::API)
    }

    fn all_shows(&self) -> ClientResult<String> {
        self.client
            .shows(vec![
                ("page-size", "50"),
                ("platform-slug", "partnerplayer"),
            ])
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
