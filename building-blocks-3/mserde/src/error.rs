use core::fmt;
use std::fmt::Display;

use serde::{de, ser};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    Message(String),
    UnsupportedType,
    Eof,
    Syntax,
    TrailingCharacters,

    ExpectedBool,
    ExpectedInteger,
    ExpectedCR,
    ExpectedLF,
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Message(msg) => formatter.write_str(msg),
            Error::Eof => formatter.write_str("unexpected end of input"),
            Error::Syntax => formatter.write_str("invalid syntax"),
            Error::UnsupportedType => formatter.write_str("unexpected type"),
            _ => formatter.write_str("fuck"),
        }
    }
}

impl std::error::Error for Error {}
