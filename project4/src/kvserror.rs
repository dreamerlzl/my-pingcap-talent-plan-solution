use std::{str::Utf8Error, string::FromUtf8Error};

use thiserror::Error;

pub type Result<T> = std::result::Result<T, KvsError>;

#[derive(Error, Debug)]
pub enum KvsError {
    #[error("Key not found")]
    KeyNotFound(String),

    #[error("fail to open kvs store")]
    IoError(#[from] std::io::Error),

    #[error("fail to deserialize kvs store")]
    DeError(#[from] bson::de::Error),

    #[error("fail to serialize to kvs store")]
    SeError(#[from] bson::ser::Error),

    #[error("invalid engine {0}")]
    InvalidEngine(String),

    #[error("invalid addr {0}")]
    InvalidAddr(String),

    #[error("fail to connect to server {0}")]
    ServerConnFail(String),

    #[error("fail to serde {0}")]
    KSPSerdeError(#[from] serde_json::Error),

    #[error("unexpected response {0}")]
    RequestError(String),

    #[error("sled error")]
    Sled(#[from] sled::Error),

    #[error("fail to convert Vec<u8> into String")]
    Utf8(#[from] FromUtf8Error),
}
