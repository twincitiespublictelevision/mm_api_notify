use config::DBConfig;
use storage::error::StoreResult;

pub enum StorageStatus {
    Available,
    NotAvailable,
}

pub trait Storage<T> {
    fn new(config: Option<&DBConfig>) -> StoreResult<Self> where Self: Sized;
    fn get(&self, id: &str, item_type: &str) -> Option<StoreResult<T>>;
    fn put(&self, item: &T) -> StoreResult<StorageStatus>;
    fn updated_at(&self) -> Option<i64>;
}
