use std::io;
use std::path::PathBuf;

use crate::engine::KvsEngine;

#[derive(Debug, Error)]
pub enum SledStoreError {
    #[error(transparent)]
    IoError(#[from] io::Error),
    #[error(transparent)]
    SledError(#[from] sled::Error),
    #[error(transparent)]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
}

pub type Result<T> = std::result::Result<T, SledStoreError>;

#[derive(Debug)]
pub struct SledStore {
    directory: PathBuf,
    db: sled::Db,
}

impl SledStore {
    pub fn new() -> Result<Self> {
        let curr_dir = std::env::current_dir()?;
        Self::open(&curr_dir)
    }

    pub fn open(directory: impl Into<PathBuf>) -> Result<Self> {
        let directory: PathBuf = directory.into();

        let db = sled::open(directory.clone())?;
        Ok(SledStore { directory, db })
    }
}

impl KvsEngine for SledStore {
    type Error = SledStoreError;

    fn set(&mut self, key: String, value: String) -> Result<()> {
        self.db.insert(key.as_bytes(), value.as_bytes())?;
        Ok(())
    }

    fn get(&mut self, key: String) -> Result<Option<String>> {
        let res = self.db.get(key.as_bytes())?;
        if let Some(res) = res {
            let string = String::from_utf8(res.to_vec())?;
            Ok(Some(string))
        } else {
            Ok(None)
        }
    }

    fn remove(&mut self, key: String) -> Result<()> {
        self.db.remove(key.as_bytes())?;
        Ok(())
    }
}
