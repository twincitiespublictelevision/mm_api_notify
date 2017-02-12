extern crate mongodb;

use mongodb::db::{Database, ThreadedDatabase};

use config::Config;
use types::ThreadedAPI;

pub struct Runtime {
    pub api: ThreadedAPI,
    pub config: Config,
    pub db: Database,
    pub verbose: bool,
}
