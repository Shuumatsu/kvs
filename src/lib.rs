#![feature(with_options)]

#[macro_use]
mod utils;

use std::collections::HashMap;
use std::fs::{self, File};
use std::io;
use std::io::prelude::*;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;

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
    #[error("key {0} not exist")]
    KeyNotExist(String),
}

pub type Result<T> = std::result::Result<T, KvsError>;

#[derive(Debug)]
struct Record {
    offset: u64,
    size: u64,
}

pub struct KvStore {
    file: File,
    path: PathBuf,
    index: HashMap<String, Record>,
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
        let mut file = option.open(&path)?;

        let mut index = HashMap::new();

        let mut curr_offset = 0;
        while let Ok(doc) = bson::Document::from_reader(&mut file) {
            let next_offset = file.seek(io::SeekFrom::Current(0))?;

            let command: Command = bson::from_document(doc)?;
            match command {
                Command::Set { key, value } => index.insert(
                    key,
                    Record {
                        offset: curr_offset,
                        size: next_offset - curr_offset,
                    },
                ),
                Command::Remove { key } => index.remove(&key),
            };

            curr_offset = next_offset;
        }

        Ok(KvStore { file, path, index })
    }

    // When setting a key to a value, kvs writes the set command to disk in a sequential log,
    // then stores the log pointer (file offset) of that command in the in-memory index from key to pointer.
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let prev_offset = self.file.seek(io::SeekFrom::Current(0))?;

        let command = Command::Set {
            key: key.clone(),
            value,
        };
        let serialized = bson::to_document(&command)?;
        serialized.to_writer(&mut self.file)?;

        let curr_offset = self.file.seek(io::SeekFrom::Current(0))?;

        let record = Record {
            offset: prev_offset,
            size: curr_offset - prev_offset,
        };
        self.index.insert(key, record);

        Ok(())
    }

    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        if !self.index.contains_key(&key) {
            return Ok(None);
        }

        let curr_offset = self.file.seek(io::SeekFrom::Current(0))?;

        let record = unwrap!(Some, self.index.get(&key));
        self.file.seek(io::SeekFrom::Start(record.offset))?;

        let mut buf = vec![0; record.size as usize];
        self.file.read_exact(&mut buf)?;

        self.file.seek(io::SeekFrom::Start(curr_offset))?;

        let command = bson::Document::from_reader(&buf[..])?;
        let command: Command = bson::from_document(command)?;

        if let Command::Set { value, .. } = command {
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    // When removing a key, kvs writes the rm command in the log,
    // then removes the key from the in-memory index.
    pub fn remove(&mut self, key: String) -> Result<()> {
        if !self.index.contains_key(&key) {
            return Err(KvsError::KeyNotExist(key));
        }
        self.index.remove(&key);

        let command = Command::Remove { key };
        let serialized = bson::to_document(&command)?;

        serialized.to_writer(&mut self.file)?;

        Ok(())
    }
}
