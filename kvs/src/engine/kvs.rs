use std::collections::HashMap;
use std::fs::{self, File};
use std::io;
use std::io::prelude::*;
use std::path::PathBuf;

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::engine::KvsEngine;

#[derive(Serialize, Deserialize, Debug)]
pub enum Command {
    Set { key: String, value: String },
    Remove { key: String },
}

#[derive(Debug, Error)]
pub enum KvsStoreError {
    #[error(transparent)]
    IoError(#[from] io::Error),
    #[error(transparent)]
    EncodeError(#[from] bson::ser::Error),
    #[error(transparent)]
    DecodeError(#[from] bson::de::Error),
    #[error("key {0} not exist")]
    KeyNotExist(String),
}

pub type Result<T> = std::result::Result<T, KvsStoreError>;

#[derive(Debug)]
struct Record {
    offset: u64,
    size: u64,
}

#[derive(Debug)]
pub struct KvStore {
    file: File,
    directory: PathBuf,
    index: HashMap<String, Record>,
    commands_cnt: usize,
}

impl KvStore {
    pub fn new() -> Result<Self> {
        let curr_dir = std::env::current_dir()?;
        Self::open(&curr_dir)
    }

    pub fn open(directory: impl Into<PathBuf>) -> Result<Self> {
        let directory: PathBuf = directory.into();
        std::fs::create_dir_all(directory.clone())?;

        let (file_path, mut file) = {
            let file_path = directory.clone().join("store.kvs");

            let mut option = File::with_options();
            let option = option.create(true).read(true).write(true).append(true);
            let file = option.open(&file_path)?;

            (file_path, file)
        };

        let mut commands_cnt = 0;
        let mut index = HashMap::new();

        let mut curr_offset = 0;
        // we need additional logics? the file may not be valid
        while let Ok(doc) = bson::Document::from_reader(&mut file) {
            let next_offset = file.seek(io::SeekFrom::Current(0))?;

            let command: Command = bson::from_document(doc)?;
            commands_cnt += 1;

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

        Ok(KvStore {
            file,
            directory,
            index,
            commands_cnt,
        })
    }

    fn compact(&mut self) -> Result<()> {
        if 2 * self.index.len() > self.commands_cnt {
            return Ok(());
        }
        self.commands_cnt = self.index.len();

        let (file_path, mut file) = {
            let mut option = File::with_options();
            let option = option.create(true).read(true).write(true).append(true);

            let file_path = self.directory.as_path().join("store_bak.kvs");
            let file = option.open(file_path.clone())?;

            (file_path, file)
        };

        // maybe iter from smaller offset to greater offset is better?
        for (_, Record { offset, size }) in self.index.iter() {
            self.file.seek(io::SeekFrom::Start(*offset))?;
            let mut buf = vec![0; *size as usize];
            self.file.read_exact(&mut buf)?;
            file.write_all(&buf)?;
        }

        fs::rename(file_path, self.directory.as_path().join("store.kvs"))?;

        Ok(())
    }
}

impl KvsEngine for KvStore {
    type Error = KvsStoreError;

    // When setting a key to a value, kvs writes the set command to disk in a sequential log,
    // then stores the log pointer (file offset) of that command in the in-memory index from key to pointer.
    fn set(&mut self, key: String, value: String) -> Result<()> {
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
        self.commands_cnt += 1;

        self.compact()
    }

    fn get(&mut self, key: String) -> Result<Option<String>> {
        if !self.index.contains_key(&key) {
            return Ok(None);
        }

        let curr_offset = self.file.seek(io::SeekFrom::Current(0))?;

        let record = self.index.get(&key).unwrap();
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
    fn remove(&mut self, key: String) -> Result<()> {
        if !self.index.contains_key(&key) {
            return Err(KvsStoreError::KeyNotExist(key));
        }
        self.index.remove(&key);

        let command = Command::Remove { key };
        let serialized = bson::to_document(&command)?;

        serialized.to_writer(&mut self.file)?;
        self.commands_cnt += 1;

        self.compact()
    }
}
