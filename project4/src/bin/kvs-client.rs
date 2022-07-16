use std::net::SocketAddr;

use clap::{Parser, Subcommand};
use tracing::info;
use tracing_subscriber;

use kvs::{KvsClient, KvsError, Result};

const DEFAULT_SERVER_ADDR: &'static str = "127.0.0.1:4000";

// clap(version) adds -V option
#[derive(Parser, Debug)]
#[clap(version, about)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SC,
}

#[derive(Subcommand, Debug)]
enum SC {
    Set {
        key: String,
        value: String,
        #[clap(short, long)]
        addr: Option<String>,
    },
    Get {
        key: String,
        #[clap(short, long)]
        addr: Option<String>,
    },
    Rm {
        key: String,
        #[clap(short, long)]
        addr: Option<String>,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_line_number(true)
        .with_writer(std::io::stderr)
        .init();
    let opts = Opts::parse();

    match opts.subcmd {
        SC::Set { key, value, addr } => {
            let addr_str = addr.unwrap_or(DEFAULT_SERVER_ADDR.to_owned());
            let addr = addr_str
                .parse::<SocketAddr>()
                .map_err(|_| KvsError::InvalidAddr(addr_str))?;
            let mut client = KvsClient::new(addr)?;
            client.set(key, value)?
        }
        SC::Get { key, addr } => {
            let addr_str = addr.unwrap_or(DEFAULT_SERVER_ADDR.to_owned());
            let addr = addr_str
                .parse::<SocketAddr>()
                .map_err(|_| KvsError::InvalidAddr(addr_str))?;
            let mut client = KvsClient::new(addr)?;
            if let Some(v) = client.get(key)? {
                info!(value = v.as_str(), "the value of key is");
                println!("{}", v);
            } else {
                info!("no such key");
                println!("Key not found");
            }
        }
        SC::Rm { key, addr } => {
            let addr_str = addr.unwrap_or(DEFAULT_SERVER_ADDR.to_owned());
            let addr = addr_str
                .parse::<SocketAddr>()
                .map_err(|_| KvsError::InvalidAddr(addr_str))?;
            let mut client = KvsClient::new(addr)?;
            client.remove(key)?
        }
    };
    Ok(())
}
