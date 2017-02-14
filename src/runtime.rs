use config::Config;
use types::{StorageEngine, ThreadedAPI};

pub struct Runtime<T>
    where T: StorageEngine
{
    pub api: ThreadedAPI,
    pub config: Config,
    pub store: T,
    pub verbose: bool,
}
