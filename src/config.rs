use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use dirs::{config_dir, home_dir};
use serde_derive::{Deserialize, Serialize};
use toml;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub directory: PathBuf,
}

impl Into<PathBuf> for Config {
    fn into(self) -> PathBuf {
        self.directory
    }
}

impl FromStr for Config {
    type Err = toml::de::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let config: Config = toml::from_str(s)?;
        Ok(config)
    }
}

impl Config {
    pub fn new(directory: PathBuf) -> Self {
        Self { directory }
    }

    pub fn from_toml(toml: toml::value::Value) -> io::Result<Self> {
        let config: Config = toml::from_str(&toml.to_string())?;
        Ok(config)
    }

    pub fn get_dots_dir() -> Option<PathBuf> {
        if let Some(config_path) = Config::get_config_file() {
            let toml = crate::paths::read_file(&config_path).unwrap();

            match Config::from_str(&toml) {
                Ok(config) => Some(config.directory),
                // REVIEW: turn into Error?
                // not able to read config
                Err(_) => None,
            }
        } else {
            None
        }
    }

    /// Search $HOME and $XDG_CONFIG_HOME for badm config file path
    fn get_config_file() -> Option<PathBuf> {
        let config_file_name = ".badm.toml";

        // TODO: make this cleaner
        if home_dir().unwrap().join(config_file_name).exists() {
            Some(home_dir().unwrap().join(config_file_name))
        } else if config_dir().unwrap().join(config_file_name).exists() {
            Some(config_dir().unwrap().join(config_file_name))
        } else {
            None
        }
    }

    pub fn write_toml_config(self) -> io::Result<()> {
        // check to see if config file already exists, if not default to HOME
        let config_file_path = match Config::get_config_file() {
            Some(path) => path,
            None => home_dir().unwrap().join(".badm.toml"),
        };

        let toml = toml::to_string(&self).unwrap();
        println!("{:?}", config_file_path);

        // write to file
        let mut file = File::create(&config_file_path)?;

        file.write_all(&toml.into_bytes())?;
        file.sync_data()?;

        Ok(())
    }

    // REVIEW: how should we handle if a dotfiles directory is already set?
    pub fn set_dots_dir(path: &Path) -> io::Result<()> {
        let config = Config::new(path.to_path_buf());

        config.write_toml_config()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[ignore]
    #[test]
    fn set_dots_dir_test() -> io::Result<()> {
        let home_dir = home_dir().unwrap();
        if !home_dir.exists() {
            fs::create_dir_all(&home_dir)?;
        }
        let dots_dir = home_dir.join(".dotfiles/.badm.toml");
        Config::set_dots_dir(&dots_dir)?;

        let toml = crate::paths::read_file(&home_dir.join(".badm.toml"))?;

        // Read file contents
        let config: Config = toml::from_str(&toml)?;
        assert_eq!(config.directory, dots_dir);

        Ok(())
    }

    #[ignore]
    #[test]
    fn write_toml_config_test() -> io::Result<()> {
        let dir = home_dir().unwrap().join(".dotfiles");
        let config = Config::new(dir);
        config.write_toml_config()?;

        Ok(())
    }
}
