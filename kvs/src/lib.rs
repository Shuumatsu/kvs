#![feature(with_options)]

#[macro_use]
extern crate thiserror;

#[macro_use]
mod utils;
mod engine;

pub use engine::{KvStore, KvsEngine, KvsStoreError, SledStore, SledStoreError};
