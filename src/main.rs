mod cli;
mod config;
#[macro_use]
mod errors;

use rcon::{AsyncStdStream, Connection};

use std::borrow::Cow;
use std::env;
use std::io::{BufRead, stdin, stdout, Write};

fn main() {
    let result = async_std::task::block_on(_main());
    if let Err(e) = result {
        std::process::exit(e);
    }
}

async fn _main() -> Result<(), i32> {
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
                    if server.contains(":") {
                        errorln!("Short names must not contain colons");
                        return Err(24);
                    }

                    let address = args.value_of("address").unwrap();
                    let password = args.value_of("password").unwrap();

                    let password = if password == "-" {
                        Cow::Owned(read_external_password().map_err(|e| {
                            errorln!("Could not read password: {}", e);
                            23
                        })?)
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
    let (address, password) = if server.contains(":") {
        read_external_password().map(|p| (server, Cow::Owned(p))).map_err(|e| {
            errorln!("Could not read password: {}", e);
            23
        })?
    } else {
        config.get(server).map(|(a, p)| (a, p.into())).ok_or_else(|| {
            errorln!("Server {} is not configured", server);
            3
        })?
    };

    let mut conn = <Connection<AsyncStdStream>>::builder()
        .enable_minecraft_quirks(true)
        .connect(address, &password)
        .await
        .map_err(|e| {
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

    println!("{}", conn.cmd(&command).await.map_err(|e| {
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

/// Read the password from an external source.
///
/// First it tries to use the environment variable RCONC_SERVER_PASSWORD.
/// If it is not set, the password is read from stdin up to first newline/eof.
fn read_external_password() -> errors::Result<String> {
    if let Some(password) = env::var_os("RCONC_SERVER_PASSWORD") {
        return password.into_string()
            .map_err(|_| "Password is not a valid utf-8 string".into());
    }

    print!("Enter password: ");
    stdout().flush().expect("Could not flush stdout");

    let stdin = stdin();
    let password = stdin.lock().lines().next();

    if let Some(Ok(password)) = password {
        Ok(password)
    } else {
        Err("Could not read password from stdin".into())
    }
}
