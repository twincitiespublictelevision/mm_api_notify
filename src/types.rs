extern crate chrono;
extern crate mm_client;

use chrono::Duration;

use client::APIClient;
use objects::Object;
use storage::Storage;

pub type ImportResult = (i64, i64);
pub type RunResult = (Duration, ImportResult);

pub trait ThreadedAPI: APIClient + Sync {}
impl<T: APIClient + Sync> ThreadedAPI for T {}

pub trait StorageEngine: Storage<Object> + Sync {}
impl<T: Storage<Object> + Sync> StorageEngine for T {}
