extern crate chrono;
extern crate mm_client;

use chrono::Duration;
use mm_client::Client;

use std::sync::Arc;

use storage::Store;

pub type ImportResult = (i64, i64);
pub type RunResult = (Duration, ImportResult);
pub type ThreadedStore = Arc<Store>;
pub type ThreadedAPI = Arc<Client>;
