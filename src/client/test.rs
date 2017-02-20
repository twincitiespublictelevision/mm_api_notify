extern crate mm_client;

use mm_client::MMCError;

use std::sync::{Arc, Mutex};

use client::client::APIClient;
use client::error::{ClientError, ClientResult};
use config::APIConfig;

#[derive(Debug)]
pub struct TestClient {
    response: Option<String>,
    pub reqs: Arc<Mutex<Vec<String>>>,
}

impl TestClient {
    pub fn set_response(&mut self, response: String) {
        self.response = Some(response)
    }
}

impl APIClient for TestClient {
    fn new(_: Option<&APIConfig>) -> ClientResult<TestClient> {
        Ok(TestClient {
            response: None,
            reqs: Arc::new(Mutex::new(vec![])),
        })
    }

    fn url(&self, url: &str) -> ClientResult<String> {
        let mut reqs = self.reqs.lock().unwrap();
        reqs.push(url.to_string());
        self.response.clone().ok_or(ClientError::API(MMCError::ResourceNotFound))
    }

    fn all_shows(&self) -> ClientResult<String> {
        self.url("all_shows")
    }

    fn changes(&self, _: &str) -> ClientResult<String> {
        self.url("changes")
    }
}
