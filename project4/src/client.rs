use std::{
    io::{BufReader, Write},
    net::{SocketAddr, TcpStream},
};

use crate::{transmit::to_bytes, KvsError, Response, Result, KSP};

use serde::Deserialize;
use serde_json::de::{Deserializer, IoRead};
use tracing::info;

pub struct KvsClient {
    addr: SocketAddr,
    stream: TcpStream,
    reader: Deserializer<IoRead<BufReader<TcpStream>>>,
}

impl KvsClient {
    pub fn new(addr: SocketAddr) -> Result<Self> {
        info!(addr = format!("{:?}", &addr).as_str(), "connecting to");
        let stream = TcpStream::connect(addr)
            .map_err(|_| KvsError::ServerConnFail(format!("{:?}", addr)))?;
        let reader = Deserializer::from_reader(BufReader::new(stream.try_clone()?));
        Ok(Self {
            addr,
            stream,
            reader,
        })
    }

    pub fn set(&mut self, key: String, val: String) -> Result<()> {
        info!(key = key.as_str(), val = val.as_str(), "client set");
        self.send_request(KSP::Set(key, val))?;
        info!("client waiting for set resp");
        match self.get_response()? {
            Response::Ok(()) => Ok(()),
            Response::Err(s) => Err(KvsError::RequestError(s)),
            _ => panic!("unexpected resp type"),
        }
    }

    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        info!(key = key.as_str(), "client get");
        self.send_request(KSP::Get(key))?;
        info!("client waiting for get resp");
        match self.get_response()? {
            Response::OkWith(s) => Ok(s),
            Response::Err(s) => Err(KvsError::RequestError(s)),
            _ => panic!("unexpected resp type"),
        }
    }

    pub fn remove(&mut self, key: String) -> Result<()> {
        info!(key = key.as_str(), "client remove");
        self.send_request(KSP::Rm(key))?;
        info!("client waiting for resp");
        match self.get_response()? {
            Response::Ok(()) => Ok(()),
            Response::Err(s) => Err(KvsError::RequestError(s)),
            _ => panic!("unexpected resp type"),
        }
    }

    fn send_request(&mut self, request: KSP) -> Result<()> {
        let bytes = to_bytes(request)?;
        self.stream
            .write_all(bytes.as_slice())
            .map_err(KvsError::IoError)
    }

    fn get_response(&mut self) -> Result<Response> {
        Response::deserialize(&mut self.reader).map_err(KvsError::KSPSerdeError)
    }
}
