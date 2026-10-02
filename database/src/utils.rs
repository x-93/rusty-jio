//! Serialization utilities for database encoding.

use super::errors::{StoreError, StoreResult};
use borsh::{BorshDeserialize, BorshSerialize};

pub fn serialize_to_vec<T: BorshSerialize>(item: &T) -> StoreResult<Vec<u8>> {
    borsh::to_vec(item).map_err(|e| StoreError::SerializationError(e.to_string()))
}

pub fn deserialize_from_slice<T: BorshDeserialize>(bytes: &[u8]) -> StoreResult<T> {
    T::try_from_slice(bytes).map_err(|e| StoreError::SerializationError(e.to_string()))
}
