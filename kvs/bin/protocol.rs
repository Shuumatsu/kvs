use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Request {
    Get { key: String },
    Set { key: String, value: String },
    Remove { key: String },
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Response {
    Success(Option<String>),
    Failed(String),
}
