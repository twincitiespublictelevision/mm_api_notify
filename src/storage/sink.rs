use config::DBConfig;
use objects::Object;
use storage::error::{StoreError, StoreResult};
use storage::storage::Storage;

#[derive(Debug)]
pub struct SinkStore {}

impl Storage<Object> for SinkStore {
    fn new(config: Option<&DBConfig>) -> StoreResult<SinkStore> {
        Ok(SinkStore {})
    }

    fn get(&self, id: &str, obj_type: &str) -> Option<Object> {
        None
    }

    fn put(&self, item: &Object) -> StoreResult<Object> {
        Err(StoreError::StorageWriteError)
    }

    fn updated_at(&self) -> Option<i64> {
        None
    }
}
