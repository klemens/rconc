mod cli;
mod config;
#[macro_use]
mod errors;

use rcon::{AsyncStdStream, Connection};

use std::borrow::Cow;
use std::env;
use std::io::{stdin, stdout, BufRead, Write};

fn main() {
    let result = async_std::task::block_on(_main());
    if let Err(e) = result {
        std::process::exit(e);
    }
}

async fn _main() -> Result<(), i32> {
    let args = cli::parse_cli();

    let mut config = config::Config::load().map_err(|e| {
        eprintln!("Error while loading config: {:?}", e);
        2
    })?;

    // server management (adding, removing, listing)
    if let Some(("server", args)) = args.subcommand() {
        match args.subcommand() {
            Some(("add", args)) => {
                let server: String = args.get_one::<String>("name").unwrap().clone();
                if config.get(&server).is_none() {
                    if server.contains(':') {
                        eprintln!("Short names must not contain colons");
                        return Err(24);
                    }

                    let address: String = args.get_one::<String>("address").unwrap().clone();
                    let password: String = args.get_one::<String>("password").unwrap().clone();

                    let password = if password == "-" {
                        Cow::Owned(read_external_password().map_err(|e| {
                            eprintln!("Could not read password: {}", e);
                            23
                        })?)
                    } else {
                        Cow::Borrowed(&password)
                    };

                    config.set(&server, &address, &password);
                    config.save().map_err(|e| {
                        eprintln!("Could not save the config file: {}", e);
                        21
                    })?;
                } else {
                    eprintln!("Server {} already exists", server);
                    return Err(20);
                }
            }
            Some(("remove", args)) => {
                let server: String = args.get_one::<String>("name").unwrap().clone();
                if config.get(&server).is_some() {
                    config.remove(&server);
                    config.save().map_err(|e| {
                        eprintln!("Could not save the config file: {}", e);
                        21
                    })?;
                } else {
                    eprintln!("Server {} does not exist", server);
                    return Err(22);
                }
            }
            Some(("list", args)) => {
                for server in config.servers() {
                    if let Some((address, password)) = config.get(server) {
                        if args.contains_id("show-passwords") {
                            println!("{}: {} ({})", server, address, password);
                        } else {
                            println!("{}: {}", server, address);
                        }
                    }
                }
            }
            _ => unreachable!(),
        }
        return Ok(());
    }

    let server: String = args.get_one::<String>("server").unwrap().clone();
    let (address, password): (String, String) = if server.contains(':') {
        read_external_password()
            .map(|p| (server.clone(), p))
            .map_err(|e| {
                eprintln!("Could not read password: {}", e);
                23
            })?
    } else {
        config
            .get(&server)
            .map(|(a, p)| (a.into(), p.into()))
            .ok_or_else(|| {
                eprintln!("Server {} is not configured", server);
                3
            })?
    };

    let mut conn = <Connection<AsyncStdStream>>::builder()
        .enable_minecraft_quirks(true)
        .connect(&address, &password)
        .await
        .map_err(|e| match e {
            rcon::Error::Auth => {
                eprintln!("The server rejected our password");
                11
            }
            _ => {
                eprintln!("Could not connect to {}: {}", address, e);
                10
            }
        })?;

    if let Some(commands) = args.get_many::<String>("command") {
        let command = itertools::join(commands, " ");

        execute_command(&mut conn, &command).await?;
    } else {
        println!("No command specified, using interactive shell");
        println!("Use CTRL + C to exit");
        let stdin = stdin();
        let mut stdout = stdout();
        let mut buf = String::new();

        loop {
            print!("{} > ", &server);
            stdout.flush().unwrap();
            stdin.read_line(&mut buf).map_err(|_| 15)?;

            execute_command(&mut conn, buf.trim()).await?;
            buf.clear();
        }
    }

    Ok(())
}

async fn execute_command(conn: &mut Connection<AsyncStdStream>, cmd: &str) -> Result<(), i32> {
    println!(
        "{}",
        conn.cmd(cmd).await.map_err(|e| match e {
            rcon::Error::CommandTooLong => {
                eprintln!("The given command is too long");
                13
            }
            _ => {
                eprintln!("Could not execute command {}: {}", cmd, e);
                12
            }
        })?
    );
    Ok(())
}

/// Read the password from an external source.
///
/// First it tries to use the environment variable RCONC_SERVER_PASSWORD.
/// If it is not set, the password is read from stdin up to first newline/eof.
fn read_external_password() -> errors::Result<String> {
    if let Some(password) = env::var_os("RCONC_SERVER_PASSWORD") {
        return password
            .into_string()
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
