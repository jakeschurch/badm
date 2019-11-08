//! Contains integration tests for the badm_core crate

use std::fs;
use std::io;
use std::path::PathBuf;

use badm_core::{self, FileHandler};
use once_cell::sync::Lazy;

use crate::common::{mock_config_file, mock_dotfile_in, DOTFILES_DIR, HOME_DIR};

pub static STOW_DIR: Lazy<PathBuf> =
    Lazy::new(|| badm_core::paths::join_full_paths(&DOTFILES_DIR, &HOME_DIR).unwrap());

#[ignore]
#[test]
fn store_dotfiles_test() -> io::Result<()> {
    mock_config_file()?;

    let dotfile_path = mock_dotfile_in(HOME_DIR.to_path_buf())?;

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
    let dotfile_path = mock_dotfile_in(STOW_DIR.to_path_buf())?;

    let stripped_dotfile_path = dotfile_path
        .strip_prefix(DOTFILES_DIR.to_path_buf())
        .unwrap();

    let symlink_path = PathBuf::from("/").join(stripped_dotfile_path);
    fs::create_dir_all(symlink_path.parent().unwrap())?;
    FileHandler::create_symlink(&dotfile_path, &symlink_path)?;

    let actual_dst_path = badm_core::commands::restore_dotfile(dotfile_path)?;

    assert!(!badm_core::paths::is_symlink(&symlink_path));
    assert_eq!(actual_dst_path, symlink_path);

    Ok(())
}

#[ignore]
#[test]
fn deploy_dotfile_test() -> io::Result<()> {
    mock_config_file()?;

    // mock the stowed dotfile
    let dotfile_path = mock_dotfile_in(STOW_DIR.to_path_buf())?;

    let stripped_dotfile_path = dotfile_path
        .strip_prefix(DOTFILES_DIR.to_path_buf())
        .expect("Not able to strip prefix");

    assert!(dotfile_path.exists());

    let expected_symlink_path = PathBuf::from("/").join(stripped_dotfile_path);

    badm_core::commands::deploy_dotfile(&dotfile_path, &expected_symlink_path)?;

    assert_eq!(fs::read_link(expected_symlink_path)?, dotfile_path);

    Ok(())
}
