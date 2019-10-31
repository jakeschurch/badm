use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use dirs::{config_dir, home_dir};
use serde_derive::{Deserialize, Serialize};
use toml;

// TODO: create a util/paths mod

pub fn read_file(path: &Path) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

pub fn expand_tilde(path: &Path) -> io::Result<PathBuf> {
    if !path.starts_with("~") {
        return Ok(path.to_path_buf());
    };

    path = path
        .strip_prefix("~")
        .expect("Could not strip tilde from path!");

    Ok(home_dir().unwrap().join(path))
}

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
            let toml = read_file(&config_path).unwrap();

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

        Ok(())
    }

    // REVIEW: how should we handle if a dotfiles directory is already set?
    pub fn set_dots_dir(path: &Path) -> io::Result<()> {
        // TODO: lift this to fn normalize_path
        let dir_path = if path.starts_with("~") {
            expand_tilde(path)?
        } else if path.is_relative() {
            path.canonicalize()?
        } else {
            path.to_path_buf()
        };

        let config = Config::new(dir_path);

        config.write_toml_config()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[ignore]
    #[test]
    fn set_dots_dir_test() -> io::Result<()> {
        let home_dir = home_dir().unwrap();
        if !home_dir.exists() {
            fs::create_dir_all(&home_dir)?;
        }

        let dots_dir = tempdir()?.into_path();
        Config::set_dots_dir(&dots_dir)?;

        // Load config into Config struct to ensure directory set is correct
        let expected_config_path = home_dir.join(".badm.toml");
        let toml = read_file(&expected_config_path)?;

        let config: Config = toml::from_str(&toml)?;
        assert_eq!(config.directory, dots_dir);

        Ok(())
    }

    #[ignore]
    #[test]
    fn write_toml_config_test() -> io::Result<()> {
        let dir = tempdir()?.into_path();
        let config = Config::new(dir);
        config.write_toml_config()?;

        Ok(())
    }
}
