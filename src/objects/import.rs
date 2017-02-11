extern crate mongodb;
extern crate serde_json;

use self::mongodb::db::{Database, ThreadedDatabase};
use self::serde_json::Value as Json;

use error::IngestResult;
use types::ImportResult;
use types::ThreadedAPI;

pub trait Importable {
    type Value;

    fn import(&self,
              api: &ThreadedAPI,
              db: &Database,
              follow_refs: bool,
              since: i64)
              -> ImportResult;
    fn from_json(json: &Json) -> IngestResult<Self::Value>;
}
