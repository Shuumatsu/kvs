mod kvs;
mod sled;

use std::error::Error;

pub trait KvsEngine {
    type Error: Error;

    fn set(&mut self, key: String, value: String) -> Result<(), Self::Error>;

    fn get(&mut self, key: String) -> Result<Option<String>, Self::Error>;

    fn remove(&mut self, key: String) -> Result<(), Self::Error>;
}

pub use self::sled::{SledStore, SledStoreError};
pub use kvs::{KvStore, KvsStoreError};
