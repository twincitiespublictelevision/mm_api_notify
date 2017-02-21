use config::DBConfig;
use objects::Object;
use storage::error::{StoreError, StoreResult};
use storage::storage::Storage;

#[derive(Debug)]
pub struct SinkStore {
    resp: Option<Object>,
}

impl SinkStore {
    pub fn set_response(&mut self, obj: Object) {
        self.resp = Some(obj);
    }
}

impl Storage<Object> for SinkStore {
    fn new(_: Option<&DBConfig>) -> StoreResult<SinkStore> {
        Ok(SinkStore { resp: None })
    }

    fn get(&self, _: &str, _: &str) -> Option<Object> {
        self.resp.clone()
    }

    fn put(&self, _: &Object) -> StoreResult<Object> {
        self.resp.clone().ok_or(StoreError::StorageWriteError)
    }

    fn updated_at(&self) -> Option<i64> {
        None
    }
}
