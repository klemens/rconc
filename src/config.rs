use app_dirs::{app_root, AppInfo, AppDataType, get_app_root};
use ini::ini::Ini;

use crate::errors::*;

use std::fs::File;

const APP_INFO: AppInfo = AppInfo {
    name: "rconc",
    author: "Klemens Schölhorn",
};

pub struct Config {
    ini: Ini,
}

impl Config {
    pub fn load() -> Result<Config> {
        let config_path = get_app_root(AppDataType::UserConfig, &APP_INFO)
            .chain_err(|| "could not find config directory")?;
        let config_file_path = config_path.join("config");

        let ini = if config_file_path.exists() {
            let mut config_file = File::open(config_file_path)
                .chain_err(|| "could not open config file")?;
            Ini::read_from(&mut config_file)
                .chain_err(|| "could not parse config file")?
        } else {
            Ini::new()
        };

        Ok(Config {
            ini: ini,
        })
    }

    pub fn save(&self) -> Result<()> {
        let config_path = app_root(AppDataType::UserConfig, &APP_INFO)
            .chain_err(|| "could not create config directory")?;
        let config_file_path = config_path.join("config");

        let mut config_file = File::create(config_file_path)
            .chain_err(|| "could not open config file")?;

        self.ini.write_to(&mut config_file)
            .chain_err(|| "could not write to the config file")
    }

    pub fn get(&self, server: &str) -> Option<(&str, &str)> {
        return match (self.ini.get_from(Some(server), "address"),
                      self.ini.get_from(Some(server), "password")) {
            (Some(addr), Some(pass)) => Some((addr, pass)),
            _ => None
        }
    }

    pub fn set(&mut self, server: &str, address: &str, password: &str) {
        self.ini.with_section(Some(server))
            .set("address", address)
            .set("password", password);
    }

    pub fn remove(&mut self, server: &str) {
        self.ini.delete(Some(server));
    }

    pub fn servers(&self) -> impl Iterator<Item = &str> {
        self.ini.sections().filter_map(std::convert::identity)
    }
}
