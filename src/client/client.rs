use client::error::ClientResult;
use config::APIConfig;

pub trait APIClient {
    fn new(config: Option<&APIConfig>) -> ClientResult<Self>
    where
        Self: Sized;
    fn url(&self, url: &str) -> ClientResult<String>;
    fn show(&self, id: &str) -> ClientResult<String>;
    fn franchise(&self, id: &str) -> ClientResult<String>;
    fn all_shows(&self) -> ClientResult<String>;
    fn changes(&self, since: &str) -> ClientResult<String>;
}
