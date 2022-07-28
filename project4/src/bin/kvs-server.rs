// kvs-server [--addr IP-PORT(string)] [--engine ENGINE-NAME(string)]
// kvs-server -V

use std::env::current_dir;
use std::fs::{self, create_dir_all};
use std::net::{AddrParseError, SocketAddr};
use std::path::Path;
use std::process::exit;

use clap::Parser;
use tracing::{error, info};
use tracing_subscriber;

use kvs::{
    KvStore, KvsEngine, KvsError, KvsServer, NaiveThreadPool, RayonThreadPool, Result,
    SharedQueueThreadPool, SledKvsEngine, ThreadPool,
};

const DEFAULT_ENGINE: &'static str = "kvs";
const DEFAULT_ADDR: &'static str = "127.0.0.1:4000";

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    addr: Option<String>, // IP:PORT

    #[clap(short, long)]
    engine: Option<String>, // either kvs or sled
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_line_number(true)
        .with_writer(std::io::stderr)
        .init();
    info!(
        version = env!("CARGO_PKG_VERSION"),
        "the version of kvs-server is"
    );
    let args = Args::parse();

    let addr_str = args.addr.unwrap_or(DEFAULT_ADDR.to_owned());
    let addr = parse_addr(&addr_str).map_err(|_| KvsError::InvalidAddr(addr_str.to_owned()))?;

    let engine = args.engine.unwrap_or(DEFAULT_ENGINE.to_owned());
    if let Some(existing_engine) = current_engine()? {
        if existing_engine != engine {
            error!("inconsistent kv engine");
            exit(1);
        }
    }
    record_current_engine(&engine)?;
    info!(
        addr = addr_str.as_str(),
        engine = engine.as_str(),
        "server runs"
    );
    match engine.as_ref() {
        "kvs" => {
            // info!("server runs with engine: kvs and addr: {}", &addr);
            let dir = Path::new("./fuck");
            create_dir_all(dir)?;
            let engine = KvStore::open(dir)?;
            run_with_engine(addr, engine)
        }
        "sled" => {
            let dir = Path::new("./fuck");
            create_dir_all(dir)?;
            let engine = SledKvsEngine::open(dir)?;
            run_with_engine(addr, engine)
        }
        _ => Err(KvsError::InvalidEngine(format!(
            "no such engine {}",
            engine
        ))),
    }
}

fn parse_addr(addr: &str) -> std::result::Result<SocketAddr, AddrParseError> {
    addr.parse::<SocketAddr>()
}

fn run_with_engine<E: KvsEngine + 'static>(addr: SocketAddr, engine: E) -> Result<()> {
    let mut server = KvsServer::new(addr, engine, RayonThreadPool::new(10)?);
    server.run()
}

fn record_current_engine(engine: &str) -> Result<()> {
    fs::write(current_dir()?.join("engine"), engine)?;
    Ok(())
}

fn current_engine() -> Result<Option<String>> {
    let path = current_dir()?.join("engine");
    if !path.exists() {
        return Ok(None);
    }
    Ok(Some(fs::read_to_string(path)?))
}
