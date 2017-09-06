extern crate serde_json;

use self::serde_json::Value as Json;

use error::IngestResult;
use runtime::Runtime;
use types::{ImportResult, StorageEngine, ThreadedAPI};

pub trait Importable {
    fn import<T: StorageEngine, S: ThreadedAPI>(
        &self,
        runtime: &Runtime<T, S>,
        follow_refs: bool,
        since: i64,
    ) -> ImportResult;
    fn from_json(json: &Json) -> IngestResult<Self>
    where
        Self: Sized;
}
