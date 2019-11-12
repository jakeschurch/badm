use std::convert::TryFrom;
use std::fs::{self, File};
use std::io;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::errors::InputError;
use dirs::{config_dir, home_dir};
use serde_derive::{Deserialize, Serialize};
use toml;

/// Handles and saves configuration variables between application calls.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Config {
    /// Path of dotfiles directory.
    pub directory: PathBuf,
}

impl Config {
    pub(crate) fn new<P: AsRef<Path>>(directory: P) -> Result<Self, InputError> {
        let directory = directory.as_ref().to_path_buf();

        if directory.is_dir() {
            Ok(Self { directory })
        } else {
            Err(InputError::BadInput {
                err: io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Input to set dots directory is invalid",
                ),
            })
        }
    }

    // REVIEW: how should we handle if a dotfiles directory is already set?
    /// Sets arg `path` at dotfiles directory, and writes TOML config file.
    ///
    /// If path is not available it will try to be created.
    pub fn set_dots_dir(path: PathBuf) -> Result<PathBuf, InputError> {
        if !path.exists() {
            fs::create_dir_all(&path).expect("could not create path");
        } else if !path.is_dir() {
            return Err(InputError::BadInput {
                err: io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Input to set dots directory is invalid",
                ),
            });
        };

        let config = Config::new(&path)?;

        config
            .write_toml_config()
            .expect("could not write toml config");
        Ok(path)
    }

    /// If config file `.badm.toml` exists, get dotfiles directory path.
    pub fn get_dots_dir() -> Option<PathBuf> {
        if let Some(config_path) = Config::get_config_file() {
            let toml = crate::paths::read_path(&config_path).unwrap();

            let config: Config = toml::from_str(&toml).expect("Not able to read config!");
            Some(config.directory)
        } else {
            None
        }
    }

    /// Search $HOME and $XDG_CONFIG_HOME for config file path.
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
        search_paths(config_file_name, vec![
            home_dir().unwrap(),
            config_dir().unwrap(),
        ])
    }

    /// Save configuration variables to config file `.badm.toml`. If file cannot be found
    /// it will be written to $HOME.
    ///
    /// Valid locations for file location include: $HOME and $XDG_CONFIG_HOME.
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
    use failure::Error;
    use std::fs;

    #[ignore]
    #[test]
    fn set_dots_dir_test() -> Result<(), Error> {
        let home_dir = home_dir().unwrap();
        if !home_dir.exists() {
            fs::create_dir_all(&home_dir).expect("could not create home dir");
        }

        let dots_dir = home_dir.join(".dotfiles");
        Config::set_dots_dir(dots_dir.clone())?;

        let toml = crate::paths::read_path(&home_dir.join(".badm.toml"))
            .expect("could not read path");

        // Read file contents
        let config: Config =
            toml::from_str(toml.as_str()).expect("could not convert toml");
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
