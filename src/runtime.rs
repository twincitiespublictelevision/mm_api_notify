use config::Config;
use types::{ThreadedAPI, ThreadedStore};

pub struct Runtime {
    pub api: ThreadedAPI,
    pub config: Config,
    pub store: ThreadedStore,
    pub verbose: bool,
}
