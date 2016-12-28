# rconc

Simple cross-platform RCON client written in rust.

Currently rconc has only been tested against a minecraft server,
but it *should* work with every RCON server.

## Build

The project can be build using cargo:

```sh
cargo build --release
```

## Usage

The following example adds the minecraft server `mycraft.example:25575` and
checks if `mobGriefing` is enabled. See `rconc --help` for more details.

```sh
$ rconc server add mycraft mycraft.example:25575 -
Enter password: rcon-password
$ rconc server list --show-passwords
mycraft: mycraft.example:25575 (rcon-password)
$ rconc mycraft gamerule mobGriefing
mobGriefing = true
```

The configured servers are stored as an ini-file in the platform-specific
config directory (eg. `$XDG_CONFIG_HOME/rconc/config` under Linux).

## Licence

This program by Klemens Schölhorn is licenced under the terms of the GPLv3.
