use std::fs;
use std::io;
use std::path::PathBuf;

use tempfile::Builder;

use badm::paths;
use badm::{self, Config};

pub fn home_dir() -> PathBuf {
    dirs::home_dir().unwrap()
}

pub fn dotfiles_dir() -> PathBuf {
    home_dir().join(".dotfiles")
}

pub fn stow_dir() -> PathBuf {
    paths::join_full_paths(&dotfiles_dir(), &home_dir()).unwrap()
}

pub fn badm_config() -> PathBuf {
    home_dir().join(".badm.toml")
}

pub fn mock_config_file() -> io::Result<()> {
    if badm_config().exists() {
        Ok(())
    } else {
        let config = Config {
            directory: dotfiles_dir(),
        };
        config.write_toml_config()
    }
}

pub fn mock_dotfile_in(parent_dir: PathBuf) -> io::Result<PathBuf> {
    let mut builder = Builder::new();

    if !parent_dir.exists() {
        fs::create_dir_all(&parent_dir)?;
    };

    let dotfile = builder.rand_bytes(6).tempfile_in(parent_dir)?;

    let (file, dotfile_path) = dotfile.keep()?;
    file.sync_data()?;

    Ok(dotfile_path)
}
