pub mod de;
mod error;
pub mod ser;
mod visitor;

//pub use de::{from_str, Deserializer};
pub use error::{Error, Result};
pub use ser::{to_bytes, Serializer};
use serde::Serialize;

#[derive(Debug, Serialize, PartialEq, Eq)]
pub enum RESP {
    SimpleString(String),
    // BulkString(None) -> Null
    BulkString(Option<Vec<u8>>),
    Error(String),
    Integer(i64),
    Array(Vec<RESP>),
}

use RESP::*;

pub fn ping_to_bytes(maybe_s: Option<String>) -> Result<Vec<u8>> {
    let resp = if let Some(s) = maybe_s {
        RESP::Array(vec![SimpleString("ping".to_owned()), SimpleString(s)])
    } else {
        RESP::Array(vec![SimpleString("ping".to_owned())])
    };
    to_bytes(&resp)
}

pub fn set_to_bytes<T: Into<String>>(k: String, v: T) -> Result<Vec<u8>> {
    let vs: String = v.into();
    to_bytes(&RESP::Array(vec![
        BulkString(Some("set".as_bytes().to_vec())),
        BulkString(Some(k.into_bytes())),
        BulkString(Some(vs.into_bytes())),
    ]))
}

pub fn get_to_bytes(k: &str) -> Result<Vec<u8>> {
    to_bytes(&RESP::Array(vec![
        BulkString(Some("get".as_bytes().to_vec())),
        BulkString(Some(k.as_bytes().to_vec())),
    ]))
}
