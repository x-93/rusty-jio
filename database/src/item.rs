use crate::errors::StoreResult;

pub trait DbItem<T: Clone + Send + Sync> {
    fn get(&self) -> StoreResult<T>;
    fn set(&self, value: T) -> StoreResult<()>;
    fn remove(&self) -> StoreResult<()>;
}
