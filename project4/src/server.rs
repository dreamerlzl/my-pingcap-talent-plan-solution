use std::{
    io::{BufReader, BufWriter, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{Arc, Mutex},
};

use serde_json::Deserializer;
use tracing::info;

use crate::{
    threadpool::ThreadPool, transmit::to_bytes, KvsEngine, KvsError, Response, Result, KSP,
};

pub struct KvsServer<E: KvsEngine, T: ThreadPool> {
    addr: SocketAddr,
    engine: Arc<Mutex<E>>,
    threadpool: T,
}

impl<E: KvsEngine + 'static, T: ThreadPool> KvsServer<E, T> {
    pub fn new(addr: SocketAddr, engine: E, threadpool: T) -> Self {
        Self {
            addr,
            engine: Arc::new(Mutex::new(engine)),
            threadpool,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        info!("server run TBD");
        let listener = TcpListener::bind(self.addr)?;
        for stream in listener.incoming() {
            let engine = self.engine.clone();
            self.threadpool.spawn(move || {
                if let Ok(s) = stream {
                    let _ = serve(engine, s);
                    info!("finish one request!");
                } else {
                    info!("fail to get stream!");
                }
            });
        }
        Ok(())
    }
}

fn send_resp(writer: &mut impl Write, resp: Response) -> Result<()> {
    info!("server begins to send back response");
    let bytes = to_bytes(resp)?;
    writer.write_all(bytes.as_slice())?;
    writer.flush().map_err(KvsError::IoError)
}

fn serve<E: KvsEngine>(engine: Arc<Mutex<E>>, stream: TcpStream) -> Result<()> {
    let reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);
    let commands = Deserializer::from_reader(reader).into_iter::<KSP>();
    for command in commands {
        let command = command?;
        info!(
            command = format!("{:?}", command).as_str(),
            "receive command"
        );
        let mut guard = engine.lock().unwrap();
        match command {
            KSP::Get(key) => match guard.get(key) {
                Ok(v) => send_resp(&mut writer, Response::OkWith(v)),
                Err(e) => send_resp(&mut writer, Response::Err(e.to_string())),
            },
            KSP::Rm(key) => match guard.remove(key) {
                Ok(_) => send_resp(&mut writer, Response::Ok(())),
                Err(e) => send_resp(&mut writer, Response::Err(e.to_string())),
            },
            KSP::Set(key, val) => match guard.set(key, val) {
                Ok(_) => send_resp(&mut writer, Response::Ok(())),
                Err(e) => send_resp(&mut writer, Response::Err(e.to_string())),
            },
        }?;
        info!("finish processing command");
    }
    Ok(())
}
