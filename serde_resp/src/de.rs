use serde::{de, Deserialize};

use crate::error::{Error, Result};

pub struct Deserializer {}

// By convention, the public API of a Serde serializer is one or more `to_abc`
// functions such as `to_string`, `to_bytes`, or `to_writer` depending on what
// Rust types the serializer is able to produce as output.
//
// This basic serializer supports only `to_string`.
pub fn from_str<T>(value: T) -> Result<String> {
    unimplemented!()
}
