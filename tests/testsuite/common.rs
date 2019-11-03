use std::fs;
use std::io;
use std::path::PathBuf;

use dirs::home_dir;
use once_cell::sync::Lazy;
use tempfile::Builder;

use badm_core::{self, Config};

pub static HOME_DIR: Lazy<PathBuf> = Lazy::new(|| home_dir().unwrap());
pub static DOTFILES_DIR: Lazy<PathBuf> = Lazy::new(|| HOME_DIR.join(".dotfiles"));
pub static BADM_CONFIG: Lazy<PathBuf> = Lazy::new(|| HOME_DIR.join(".badm.toml"));

pub fn mock_config_file() -> io::Result<()> {
    if BADM_CONFIG.to_path_buf().exists() {
        Ok(())
    } else {
        let config = Config {
            directory: DOTFILES_DIR.to_path_buf(),
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
