use storage::error::StoreResult;

pub enum StorageStatus {
    Available,
}

pub trait Storage<T> {
    fn get(&self, id: &str, item_type: &str) -> Option<StoreResult<T>>;
    fn put(&self, item: &T) -> StoreResult<StorageStatus>;
    fn updated_at(&self) -> Option<i64>;
}
