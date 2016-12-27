use clap::{App, Arg, ArgMatches};

pub fn parse_cli<'a>() -> ArgMatches<'a> {
    App::new("rconc")
        .version(crate_version!())
        .arg(Arg::with_name("server")
                 .required(true))
        .arg(Arg::with_name("command")
                 .required(true)
                 .multiple(true))
        .get_matches()
}
