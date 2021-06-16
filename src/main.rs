#[macro_use]
extern crate clap;
use clap::{AppSettings, Clap};
use kvs::{KvStore, KvsError, Result};

#[derive(Clap)]
#[clap(version = crate_version!(), setting = AppSettings::ColoredHelp)]
struct Opts {
    // /// A level of verbosity, and can be used multiple times
    // #[clap(short, long, parse(from_occurrences))]
    // verbose: i32,
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    Set(Set),
    Get(Get),
    Rm(Rm),
}

/// Set the value of a string key to a string
#[derive(Clap)]
struct Set {
    key: String,
    value: String,
}

/// Get the string value of a given string key
#[derive(Clap)]
struct Get {
    key: String,
}

/// Remove a given key
#[derive(Clap)]
struct Rm {
    key: String,
}

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();

    let mut kvs = KvStore::new()?;

    // You can handle information about subcommands by requesting their matches by name
    // (as below), requesting just the name used, or both at the same time
    match opts.subcmd {
        SubCommand::Set(set) => kvs.set(set.key, set.value),
        // SubCommand::Get(get) => println!("{:?}", kvs.get(get.key)),
        // SubCommand::Rm(rm) => kvs.remove(rm.key),
        _ => unimplemented!(),
    }

    // more program logic goes here...
}
