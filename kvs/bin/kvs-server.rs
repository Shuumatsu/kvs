#[macro_use]
extern crate thiserror;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate anyhow;

mod protocol;

use protocol::Request;

use std::borrow::BorrowMut;
use std::error::Error;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::str::FromStr;

use anyhow::{Context, Result};
use clap::{AppSettings, Clap};
use kvs::{KvStore, KvsEngine, SledStore};
use serde::Deserialize;
use tracing::{
    debug_span, error, event, info, info_span, instrument, span, trace, trace_span, Level,
};
use tracing_subscriber::prelude::*;

use crate::protocol::Response;

#[derive(Clap, Debug)]
#[clap(name = "kvs-server", version = crate_version!(), setting = AppSettings::ColoredHelp)]
struct Opts {
    // /// A level of verbosity, and can be used multiple times
    // #[clap(short, long, parse(from_occurrences))]
    // verbose: i32,
    #[clap(long, default_value = "127.0.0.1:4000")]
    addr: String,
    #[clap(long)]
    engine: String,
}

trait Engine {
    fn set(&mut self, key: String, value: String) -> Result<(), anyhow::Error>;

    fn get(&mut self, key: String) -> Result<Option<String>, anyhow::Error>;

    fn remove(&mut self, key: String) -> Result<(), anyhow::Error>;
}

impl<T, E> Engine for T
where
    E: Error + Send + Sync + 'static,
    T: KvsEngine<Error = E>,
{
    fn set(&mut self, key: String, value: String) -> Result<(), anyhow::Error> {
        self.set(key, value).map_err(|e| e.into())
    }

    fn get(&mut self, key: String) -> Result<Option<String>, anyhow::Error> {
        self.get(key).map_err(|e| e.into())
    }

    fn remove(&mut self, key: String) -> Result<(), anyhow::Error> {
        self.remove(key).map_err(|e| e.into())
    }
}

fn handle_connection(engine: &mut dyn Engine, mut stream: TcpStream) -> Result<()> {
    let span = info_span!("handle_connection");
    let _enter = span.enter();

    info!(req_addr = stream.peer_addr()?.to_string().as_str());

    let mut de = serde_json::Deserializer::from_reader(&mut stream);
    let command = Request::deserialize(&mut de)?;
    info!(command = format!("{:?}", command).as_str());

    let resp = match command {
        Request::Get { key } => match engine.get(key) {
            Err(err) => Response::Failed(err.to_string()),
            Ok(value) => Response::Success(value),
        },
        Request::Set { key, value } => match engine.set(key, value) {
            Err(err) => Response::Failed(err.to_string()),
            Ok(()) => Response::Success(None),
        },
        Request::Remove { key } => match engine.remove(key) {
            Err(err) => Response::Failed(err.to_string()),
            Ok(()) => Response::Success(None),
        },
    };

    serde_json::to_writer(&stream, &resp)?;

    Ok(())
}

// #[instrument]
fn main() -> Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    let opts: Opts = Opts::parse();
    let addr = SocketAddr::from_str(&opts.addr)?;
    info!(
        addr = addr.to_string().as_str(),
        engine = opts.engine.as_str()
    );

    let mut engine: Box<dyn Engine> = match opts.engine.as_str() {
        "kvs" => Box::new(KvStore::new().context("failed to initialize KvStore engine")?),
        "sled" => Box::new(SledStore::new().context("failed to initialize Sled engine")?),
        engine => bail!("unsupported engine {}", engine),
    };

    let listener = TcpListener::bind(addr)?;
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Err(err) = handle_connection(&mut *engine, stream) {
                    error!(error = format!("{:?}", err).as_str());
                }
            }
            Err(err) => error!(error = format!("{:?}", err).as_str()),
        }
    }

    Ok(())
}
