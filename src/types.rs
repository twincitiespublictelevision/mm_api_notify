extern crate chrono;
extern crate mm_client;

use chrono::Duration;
use mm_client::Client;

use std::sync::Arc;

use objects::Object;
use storage::{MongoStore, Storage};

pub type ImportResult = (i64, i64);
pub type RunResult = (Duration, ImportResult);
pub type Store<T: Storage<Object>> = T;
pub type ThreadedStore = Arc<Store<MongoStore>>;
pub type ThreadedAPI = Arc<Client>;
