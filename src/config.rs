use app_dirs::{app_root, AppInfo, AppDataType};
use ini::Ini;

use errors::*;

const APP_INFO: AppInfo = AppInfo {
    name: "rconc",
    author: "Klemens Schölhorn",
};

pub struct Config {
    ini: Ini,
}

impl Config {
    pub fn load() -> Result<Config> {
        let config_path = try!(app_root(AppDataType::UserConfig, &APP_INFO)
            .chain_err(|| "could not create config directory"));

        let config_file = config_path.join("config");

        let ini = if config_file.exists() {
            try!(Ini::load_from_file(&config_file.to_string_lossy())
                .chain_err(|| "could not open config file"))
        } else {
            Ini::new()
        };

        Ok(Config {
            ini: ini,
        })
    }

    pub fn get(&self, server: &str) -> Option<(&str, &str)> {
        return match (self.ini.get_from(Some(server), "address"),
                      self.ini.get_from(Some(server), "password")) {
            (Some(addr), Some(pass)) => Some((addr, pass)),
            _ => None
        }
    }
}
