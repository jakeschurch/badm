//! Contains integration tests for the badm_core crate
use badm_core::{self, Config, FileHandler};

use std::fs;
use std::io;
use std::path::PathBuf;

use dirs::home_dir;
use once_cell::sync::Lazy;
use tempfile::Builder;

static HOME_DIR: Lazy<PathBuf> = Lazy::new(|| home_dir().unwrap());
static DOTFILES_DIR: Lazy<PathBuf> = Lazy::new(|| HOME_DIR.join(".dotfiles"));
static STOW_DIR: Lazy<PathBuf> =
    Lazy::new(|| badm_core::paths::join_full_paths(&DOTFILES_DIR, &HOME_DIR).unwrap());

fn mock_config_file() -> io::Result<()> {
    if HOME_DIR.join(".badm.toml").exists() {
        Ok(())
    } else {
        let config = Config {
            directory: DOTFILES_DIR.to_path_buf(),
        };
        config.write_toml_config()
    }
}

fn mock_dotfile(parent_dir: PathBuf) -> io::Result<PathBuf> {
    let mut builder = Builder::new();

    if !parent_dir.exists() {
        fs::create_dir_all(&parent_dir)?;
    };

    let dotfile = builder.rand_bytes(6).tempfile_in(parent_dir)?;

    let (_, dotfile_path) = dotfile.keep()?;

    Ok(dotfile_path)
}

#[ignore]
#[test]
fn store_dotfiles_test() -> io::Result<()> {
    mock_config_file()?;

    let dotfile_path = mock_dotfile(HOME_DIR.to_path_buf())?;

    let expected_stow_path = STOW_DIR.join(dotfile_path.file_name().unwrap());

    let stow_path = badm_core::commands::store_dotfile(&dotfile_path)?;

    assert!(expected_stow_path.exists());
    assert_eq!(expected_stow_path, stow_path);

    Ok(())
}

#[ignore]
#[test]
fn restore_dotfile_test() -> io::Result<()> {
    mock_config_file()?;

    // mock dotfile and corresponding symlink
    let dotfile_path = mock_dotfile(STOW_DIR.to_path_buf())?;

    let stripped_dotfile_path = dotfile_path
        .strip_prefix(DOTFILES_DIR.to_path_buf())
        .unwrap();

    let symlink_path = PathBuf::from("/").join(stripped_dotfile_path);
    fs::create_dir_all(symlink_path.parent().unwrap())?;
    FileHandler::create_symlink(&dotfile_path, &symlink_path)?;

    badm_core::commands::restore_dotfile(dotfile_path)?;

    assert!(!badm_core::paths::is_symlink(&symlink_path)?);

    Ok(())
}

#[ignore]
#[test]
fn deploy_dotfile_test() -> io::Result<()> {
    mock_config_file()?;

    // mock the stowed dotfile
    let dotfile_path = mock_dotfile(STOW_DIR.to_path_buf())?;

    let stripped_dotfile_path = dotfile_path
        .strip_prefix(DOTFILES_DIR.to_path_buf())
        .expect("Not able to strip prefix");

    assert!(dotfile_path.exists());

    let expected_symlink_path = PathBuf::from("/").join(stripped_dotfile_path);

    badm_core::commands::deploy_dotfile(&dotfile_path, &expected_symlink_path)?;

    println!("deploy ran successfully",);

    assert_eq!(fs::read_link(expected_symlink_path)?, dotfile_path);

    Ok(())
}
