use thiserror::Error;

#[derive(Error, Debug)]
pub enum MBytesError {
    #[error("invalid simple string resp {0:?}")]
    InvalidSimple(Vec<u8>),

    #[error("containing not utf-8 chars")]
    NotSimple(#[from] std::string::FromUtf8Error),

    #[error("invalid bulk string resp {0:?}")]
    InvalidBulk(Vec<u8>),

    #[error("invalid resp from redis server {0:?}")]
    InvalidResp(Vec<u8>),
}
