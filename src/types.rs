extern crate chrono;
extern crate mm_client;

use chrono::Duration;
use mm_client::Client;

use std::sync::Arc;

pub type ThreadedAPI = Arc<Client<'static>>;
pub type ImportResult = (i64, i64);
pub type RunResult = (Duration, ImportResult);
