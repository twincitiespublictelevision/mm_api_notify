extern crate chrono;
extern crate mm_client;

use chrono::Duration;
use mm_client::Client;

use std::sync::Arc;

use objects::Object;
use storage::Storage;

pub type ImportResult = (i64, i64);
pub type RunResult = (Duration, ImportResult);
pub type ThreadedAPI = Arc<Client>;

pub trait StorageEngine: Storage<Object> + Sync {}
impl<T: Storage<Object> + Sync> StorageEngine for T {}
