use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use dirs::{config_dir, home_dir};
use serde_derive::{Deserialize, Serialize};
use toml;

/// TEMP: Currently, the code sources the location of the dotfiles directory set by BADM
/// in the env variable "BADM_DIR." This will be replaced in favor of a TOML configuration
/// file in a future release.
pub const BADM_DIR_VAR: &str = "BADM_DIR";

// TODO: create a fs_utils file

pub fn read_file<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let mut file = File::open(path.as_ref())?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

pub fn expand_tilde<P: AsRef<Path>>(path: P) -> io::Result<PathBuf> {
    let mut path = path.as_ref();

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
    directory: PathBuf,
}

impl Config {
    fn new(directory: PathBuf) -> Self {
        Self { directory }
    }

    pub fn get_dots_dir(var_name: &'static str) -> Option<String> {
        match env::var(var_name) {
            Ok(location) => Some(location),
            Err(_) => None,
        }
    }

    /// Search $HOME and $XDG_CONFIG_HOME for badm config file path
    fn get_config_file() -> Option<PathBuf> {
        let config_file_name = ".badm.toml";

        if let Some(path) = home_dir() {
            Some(path.join(config_file_name))
        } else if let Some(path) = config_dir() {
            Some(path.join(config_file_name))
        } else {
            None
        }
    }

    // REVIEW: how should we handle if a dotfiles directory is already set?
    pub fn set_dots_dir<P: AsRef<Path>>(path: P) -> io::Result<()> {
        let path = path.as_ref();

        let dir_path = if path.starts_with("~") {
            expand_tilde(path)?
        } else if path.is_relative() {
            path.canonicalize()?
        } else {
            path.to_path_buf()
        };

        let config = Config::new(dir_path);
        let toml = toml::to_string(&config).unwrap();

        // check to see if config file already exists, if not default to XDG_CONFIG_HOME
        let config_file_path = match Config::get_config_file() {
            Some(path) => path,
            None => config_dir().unwrap().join(".badm.toml"),
        };

        // REVIEW: lift to create config file?
        //
        // write to file
        let mut file = if config_file_path.exists() {
            File::open(config_file_path)?
        } else {
            File::create(config_file_path)?
        };
        file.write_all(&toml.into_bytes())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[ignore]
    #[test]
    fn set_dots_dir_test() -> io::Result<()> {
        let dir = tempdir()?.into_path();
        Config::set_dots_dir(&dir)?;

        // Load config into Config struct to ensure directory set is correct
        let expected_config_path = home_dir().unwrap().join(".badm.toml");
        let toml = read_file(expected_config_path)?;

        let config: Config = toml::from_str(&toml)?;
        assert_eq!(config.directory, dir);

        Ok(())
    }
}
