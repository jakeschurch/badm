use std::convert::TryFrom;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::PathBuf;
use std::str::FromStr;

use crate::errors::InputError;
use dirs::{config_dir, home_dir};
use serde_derive::{Deserialize, Serialize};
use toml;

#[derive(Fail, Debug)]
pub enum ConfigError {
    #[fail(display = "Bad Dotfile Directory given: {:?}", directory)]
    BadConfigDir { directory: PathBuf },
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Config {
    pub directory: PathBuf,
}

impl Config {
    pub fn new(directory: PathBuf) -> Result<Self, ConfigError> {
        if directory.is_dir() {
            Ok(Self { directory })
        } else {
            Err(ConfigError::BadConfigDir { directory })
        }
    }

    pub fn get_dots_dir() -> Option<PathBuf> {
        if let Some(config_path) = Config::get_config_file() {
            let toml = crate::paths::read_path(&config_path).unwrap();

            match Config::from_str(&toml) {
                Ok(config) => Some(config.directory),

                // not able to read config
                Err(_) => None,
            }
        } else {
            None
        }
    }

    /// Search $HOME and $XDG_CONFIG_HOME for badm config file path
    fn get_config_file() -> Option<PathBuf> {
        let search_paths = |file_name: &str, dirs_vec: Vec<PathBuf>| -> Option<PathBuf> {
            for dir in dirs_vec.into_iter() {
                let possible_file_path = dir.join(file_name);

                if possible_file_path.exists() {
                    return Some(possible_file_path);
                };
            }
            None
        };

        let config_file_name = ".badm.toml";
        search_paths(
            config_file_name,
            vec![home_dir().unwrap(), config_dir().unwrap()],
        )
    }

    pub fn write_toml_config(self) -> io::Result<()> {
        // check to see if config file already exists, if not default to HOME
        let config_file_path = match Config::get_config_file() {
            Some(path) => path,
            None => home_dir().unwrap().join(".badm.toml"),
        };

        let toml = toml::to_string(&self).unwrap();
        let mut file = File::create(config_file_path)?;

        file.write_all(&toml.into_bytes())?;
        file.sync_data()?;

        Ok(())
    }

    // REVIEW: how should we handle if a dotfiles directory is already set?
    pub fn set_dots_dir(path: PathBuf) -> io::Result<()> {
        let config = Config { directory: path };

        config.write_toml_config()
    }
}

impl TryFrom<File> for Config {
    type Error = InputError;
    fn try_from(file: File) -> Result<Config, Self::Error> {
        let mut file = file;

        let contents = crate::paths::read_file(&mut file)?;

        Ok(toml::from_str(&contents)?)
    }
}

impl TryFrom<PathBuf> for Config {
    type Error = InputError;
    fn try_from(path: PathBuf) -> Result<Config, Self::Error> {
        let file = File::open(path)?;
        Config::try_from(file)
    }
}

impl FromStr for Config {
    type Err = toml::de::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let config: Config = toml::from_str(s)?;
        Ok(config)
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
        let dots_dir = home_dir.join(".dotfiles");
        Config::set_dots_dir(dots_dir.clone())?;

        let toml = crate::paths::read_path(&home_dir.join(".badm.toml"))?;

        // Read file contents
        let config: Config = toml::from_str(toml.as_str())?;
        assert_eq!(config.directory, dots_dir);

        Ok(())
    }

    #[ignore]
    #[test]
    fn write_toml_config_test() -> io::Result<()> {
        let config_path = home_dir().unwrap().join(".badm.toml");

        let dots_dir = home_dir().unwrap().join(".dotfiles");
        let expected_config = Config {
            directory: dots_dir,
        };

        let config = expected_config.clone();
        config.write_toml_config()?;

        let actual_config = match Config::try_from(config_path) {
            Ok(config) => config,
            Err(_) => Config {
                directory: PathBuf::from("/"),
            },
        };

        assert_eq!(expected_config, actual_config);

        Ok(())
    }
}
