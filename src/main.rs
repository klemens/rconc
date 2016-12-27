extern crate app_dirs;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate error_chain;
extern crate ini;
extern crate itertools;
extern crate rcon;

mod cli;
mod config;
#[macro_use]
mod errors;

use std::io::Write;

fn main() {
    if let Err(e) = _main() {
        std::process::exit(e);
    }
}

fn _main() -> Result<(), i32> {
    let args = cli::parse_cli();

    let config = config::Config::load().map_err(|e| {
        errorln!("Error while loading config: {:?}", e);
        2
    })?;

    let server = args.value_of("server").unwrap();
    let (address, password) = config.get(server).ok_or_else(|| {
        errorln!("Server {} is not configured", server);
        3
    })?;

    let mut conn = rcon::Connection::connect(address, password).map_err(|e| {
        match e {
            rcon::Error::Auth => {
                errorln!("The server rejected our password");
                11
            }
            _ => {
                errorln!("Could not connect to {}: {}", address, e);
                10
            }
        }
    })?;

    let command = itertools::join(args.values_of("command").unwrap(), " ");

    println!("{}", conn.cmd(&command).map_err(|e| {
        match e {
            rcon::Error::CommandTooLong => {
                errorln!("The given command is too long");
                13
            }
            _ => {
                errorln!("Could not execute command {}: {}", command, e);
                12
            }
        }
    })?);

    Ok(())
}
