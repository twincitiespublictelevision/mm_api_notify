use config::DBConfig;
use storage::error::StoreResult;

pub trait Storage<T>
    where Self: Sized
{
    fn new(config: Option<&DBConfig>) -> StoreResult<Self>;
    fn get(&self, id: &str, item_type: &str) -> Option<T>;
    fn put(&self, item: &T) -> StoreResult<T>;
    fn updated_at(&self) -> Option<i64>;
}
