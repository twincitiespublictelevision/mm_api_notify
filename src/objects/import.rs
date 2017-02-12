extern crate serde_json;

use self::serde_json::Value as Json;

use error::IngestResult;
use runtime::Runtime;
use types::ImportResult;

pub trait Importable {
    type Value;

    fn import(&self, runtime: &Runtime, follow_refs: bool, since: i64) -> ImportResult;
    fn from_json(json: &Json) -> IngestResult<Self::Value>;
}
