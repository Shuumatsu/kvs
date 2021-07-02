#[macro_use]
extern crate clap;
#[macro_use]
extern crate anyhow;

mod protocol;

use protocol::{Request, Response};

use std::io::Read;
use std::net::{SocketAddr, TcpStream};
use std::str::FromStr;

use anyhow::{Context, Result};
use clap::{AppSettings, Clap};
use kvs::{KvStore, KvsEngine, SledStore};
use tracing::{event, info, instrument, Level};

#[derive(Clap, Debug)]
#[clap(name = "kvs-server", version = crate_version!(), setting = AppSettings::ColoredHelp)]
struct Opts {
    // /// A level of verbosity, and can be used multiple times
    // #[clap(short, long, parse(from_occurrences))]
    // verbose: i32,
    #[clap(subcommand)]
    subcmd: SubCommand,
    #[clap(long, default_value = "127.0.0.1:4000")]
    addr: String,
}

#[derive(Clap, Debug)]
enum SubCommand {
    Set(Set),
    Get(Get),
    Rm(Rm),
}

/// Set the value of a string key to a string
#[derive(Clap, Debug)]
struct Set {
    key: String,
    value: String,
}

/// Get the string value of a given string key
#[derive(Clap, Debug)]
struct Get {
    key: String,
}

/// Remove a given key
#[derive(Clap, Debug)]
struct Rm {
    key: String,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    let opts: Opts = Opts::parse();
    let addr = SocketAddr::from_str(&opts.addr)?;
    let req = match opts.subcmd {
        SubCommand::Set(set) => Request::Set {
            key: set.key,
            value: set.value,
        },
        SubCommand::Get(get) => Request::Get { key: get.key },
        SubCommand::Rm(rm) => Request::Remove { key: rm.key },
    };
    info!(
        addr = addr.to_string().as_str(),
        request = format!("{:?}", req).as_str()
    );

    let mut stream = TcpStream::connect(addr)?;

    serde_json::to_writer(&stream, &req)?;

    let resp =
        serde_json::from_reader(&stream).context("failed to receive response from server")?;
    match resp {
        Response::Success(value) => println!("{:?}", value),
        Response::Failed(err) => println!("{:?}", err),
    }

    Ok(())
}
