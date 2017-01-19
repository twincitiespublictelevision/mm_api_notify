extern crate mongodb;
extern crate serde_json;

use self::mongodb::db::{Database, ThreadedDatabase};
use self::serde_json::Value as Json;

use error::IngestResult;
use types::ThreadedAPI;

pub trait Importable {
    type Value;

    // TODO: Refactor path_from_root to be a reference that is cloned
    // on mutation. Likely needs to be place in an ARC
    fn import(&self,
              api: &ThreadedAPI,
              db: &Database,
              follow_refs: bool,
              path_from_root: &Vec<&str>,
              since: i64)
              -> ();
    fn from_json(json: &Json) -> IngestResult<Self::Value>;
}
