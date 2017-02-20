use config::APIConfig;
use client::error::ClientResult;

pub trait APIClient {
    fn new(config: Option<&APIConfig>) -> ClientResult<Self> where Self: Sized;
    fn url(&self, url: &str) -> ClientResult<String>;
    fn all_shows(&self) -> ClientResult<String>;
    fn changes(&self, since: &str) -> ClientResult<String>;
}
