#![feature(with_options)]

use std::collections::HashMap;
use std::fs::{self, File};
use std::io;
use std::io::prelude::*;
use std::path::PathBuf;

use thiserror::Error;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Command {
    Set { key: String, value: String },
    Remove { key: String },
}

#[derive(Debug, Error)]
pub enum KvsError {
    #[error(transparent)]
    IoError(#[from] io::Error),
    #[error(transparent)]
    EncodeError(#[from] bson::ser::Error),
    #[error(transparent)]
    DecodeError(#[from] bson::de::Error),
}

pub type Result<T> = std::result::Result<T, KvsError>;

struct Log {
    offset: usize,
}

pub struct KvStore {
    file: File,
    path: PathBuf,
    index: HashMap<String, Log>,
}

impl KvStore {
    pub fn new() -> Result<KvStore> {
        Self::open(std::path::Path::new("."))
    }

    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let mut path: PathBuf = path.into();
        if path.is_dir() {
            path.push(".kvs");
        }

        let mut option = File::with_options();
        let option = option.create(true).read(true).write(true).append(true);
        let file = option.open(&path)?;

        Ok(KvStore { file, path })
    }

    // When setting a key to a value, kvs writes the set command to disk in a sequential log,
    // then stores the log pointer (file offset) of that command in the in-memory index from key to pointer.
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let command = Command::Set { key, value };
        let serialized = bson::to_document(&command)?;

        serialized.to_writer(&mut self.file)?;
        Ok(())
    }

    pub fn get(&self, key: String) -> Result<Option<String>> {
        unimplemented!()
    }

    // When removing a key, kvs writes the rm command in the log,
    // then removes the key from the in-memory index.
    pub fn remove(&mut self, key: String) -> Result<()> {
        let command = Command::Remove { key };
        let serialized = bson::to_document(&command)?;

        serialized.to_writer(&mut self.file)?;
        Ok(())
    }
}
