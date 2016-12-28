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

use std::borrow::Cow;
use std::io::{BufRead, stdin, stdout, Write};

fn main() {
    if let Err(e) = _main() {
        std::process::exit(e);
    }
}

fn _main() -> Result<(), i32> {
    let args = cli::parse_cli();

    let mut config = config::Config::load().map_err(|e| {
        errorln!("Error while loading config: {:?}", e);
        2
    })?;

    // server management (adding, removing, listing)
    if let ("server", Some(args)) = args.subcommand() {
        match args.subcommand() {
            ("add", Some(args)) => {
                let server = args.value_of("name").unwrap();
                if config.get(server).is_none() {
                    let address = args.value_of("address").unwrap();
                    let password = args.value_of("password").unwrap();

                    let password = if password == "-" {
                        print!("Enter password: ");
                        stdout().flush().expect("Could not flush stdout");

                        let stdin = stdin();
                        let password = stdin.lock().lines().next();
                        if let Some(Ok(password)) = password {
                            Cow::Owned(password)
                        } else {
                            errorln!("Could not read password from stdin");
                            return Err(23)
                        }
                    } else {
                        Cow::Borrowed(password)
                    };

                    config.set(server, address, &password);
                    config.save().map_err(|e| {
                        errorln!("Could not save the config file: {}", e);
                        21
                    })?;
                } else {
                    errorln!("Server {} already exists", server);
                    return Err(20);
                }
            }
            ("remove", Some(args)) => {
                let server = args.value_of("name").unwrap();
                if config.get(server).is_some() {
                    config.remove(server);
                    config.save().map_err(|e| {
                        errorln!("Could not save the config file: {}", e);
                        21
                    })?;
                } else {
                    errorln!("Server {} does not exist", server);
                    return Err(22);
                }
            }
            ("list", Some(args)) => {
                for server in config.servers() {
                    if let Some((address, password)) = config.get(server) {
                        if args.is_present("show-passwords") {
                            println!("{}: {} ({})", server, address, password);
                        } else {
                            println!("{}: {}", server, address);
                        }
                    }
                }
            }
            _ => unreachable!()
        }
        return Ok(());
    }

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
