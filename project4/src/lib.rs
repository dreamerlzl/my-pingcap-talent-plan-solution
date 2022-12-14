mod client;
mod kvserror;
mod kvstore;
mod server;
mod sledstore;
pub mod threadpool;
mod transmit;

pub use client::KvsClient;
pub use kvserror::{KvsError, Result};
pub use kvstore::KvStore;
pub use server::KvsServer;
pub use sledstore::SledKvsEngine;
pub use threadpool::{NaiveThreadPool, RayonThreadPool, SharedQueueThreadPool, ThreadPool};

pub trait KvsEngine: Send + Clone + 'static {
    fn set(&self, key: String, value: String) -> Result<()>;
    fn get(&self, key: String) -> Result<Option<String>>;
    fn remove(&self, key: String) -> Result<()>;
}

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum KSP {
    Set(String, String),
    Get(String),
    Rm(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    OkWith(Option<String>),
    Ok(()),
    Err(String),
}
