use config::DBConfig;
use objects::Object;
use storage::error::{StoreError, StoreResult};
use storage::storage::Storage;

#[derive(Debug)]
pub struct SinkStore {}

impl Storage<Object> for SinkStore {
    fn new(_: Option<&DBConfig>) -> StoreResult<SinkStore> {
        Ok(SinkStore {})
    }

    fn get(&self, _: &str, _: &str) -> Option<Object> {
        None
    }

    fn put(&self, _: &Object) -> StoreResult<Object> {
        Err(StoreError::StorageWriteError)
    }

    fn updated_at(&self) -> Option<i64> {
        None
    }
}
