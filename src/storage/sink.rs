use config::DBConfig;
use objects::Object;
use storage::error::{StoreError, StoreResult};
use storage::storage::{Storage, StorageStatus};

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

    fn get(&self, _: &str, _: &str) -> Option<StoreResult<Object>> {
        self.resp.clone().map(|resp| Ok(resp))
    }

    fn put(&self, _: &Object) -> StoreResult<StorageStatus> {
        self.resp.clone().ok_or(StoreError::StorageWriteError).and(Ok(StorageStatus::Available))
    }

    fn updated_at(&self) -> Option<i64> {
        None
    }
}
