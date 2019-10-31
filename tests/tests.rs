use badm_core::{self, Config, FileHandler};

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use dirs::home_dir;
use tempfile::Builder;

fn dotfiles_dir() -> PathBuf {
    home_dir().unwrap().join(".dotfiles")
}

fn stow_dir() -> PathBuf {
    badm_core::paths::join_full_paths(&dotfiles_dir(), &home_dir().unwrap()).unwrap()
}

fn mock_config_file() -> io::Result<()> {
    if dotfiles_dir().join(".badm.toml").exists() {
        Ok(())
    } else {
        let config = Config::new(dotfiles_dir());
        config.write_toml_config()
    }
}

fn mock_dotfile<P: AsRef<Path>>(parent_dir: P) -> io::Result<PathBuf> {
    let mut builder = Builder::new();
    let parent_dir = parent_dir.as_ref();

    if !parent_dir.exists() {
        fs::create_dir_all(parent_dir)?;
    };

    // create parent path
    let parent_dir = builder.tempdir_in(parent_dir)?;
    let parent_path = parent_dir.into_path();

    let dotfile = builder.rand_bytes(6).tempfile_in(parent_path)?;

    let (_, dotfile_path) = dotfile.keep()?;

    Ok(dotfile_path)
}

#[ignore]
#[test]
fn unstow_dotfile_test() -> io::Result<()> {
    mock_config_file()?;

    // mock dotfile and corresponding symlink
    let dotfile_path = mock_dotfile(stow_dir())?;

    let stripped_dotfile_path = dotfile_path.strip_prefix(dotfiles_dir()).unwrap();

    let symlink_path = PathBuf::from("/").join(stripped_dotfile_path);
    fs::create_dir_all(symlink_path.parent().unwrap())?;
    FileHandler::create_symlink(&dotfile_path, &symlink_path)?;

    badm_core::unstow_dotfile(dotfile_path)?;

    assert!(!badm_core::paths::is_symlink(&symlink_path)?);

    Ok(())
}

#[ignore]
#[test]
fn stow_dotfiles_test() -> io::Result<()> {
    mock_config_file()?;

    let dotfile_path = mock_dotfile(home_dir().unwrap())?;

    let expected_stow_path =
        stow_dir().join(dotfile_path.strip_prefix(home_dir().unwrap()).unwrap());

    let stow_path = badm_core::stow_dotfile(&dotfile_path)?;

    assert_eq!(fs::read_link(dotfile_path)?, stow_path);
    assert_eq!(expected_stow_path, stow_path);

    Ok(())
}

#[ignore]
#[test]
fn create_dotfiles_symlink_test() -> io::Result<()> {
    mock_config_file()?;

    // mock the stowed dotfile
    let dotfile_path = mock_dotfile(stow_dir())?;

    let stripped_dotfile_path = dotfile_path
        .strip_prefix(dotfiles_dir())
        .expect("Not able to strip prefix");

    let expected_symlink_path = PathBuf::from("/").join(stripped_dotfile_path);

    badm_core::create_dotfile_symlink(&dotfile_path)?;

    assert_eq!(fs::read_link(expected_symlink_path)?, dotfile_path);

    Ok(())
}
