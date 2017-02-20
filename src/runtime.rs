use config::Config;
use types::{StorageEngine, ThreadedAPI};

pub struct Runtime<T, S>
    where T: StorageEngine,
          S: ThreadedAPI
{
    pub api: S,
    pub config: Config,
    pub store: T,
    pub verbose: bool,
}
