//! Contains integration tests for the badm_core crate

use std::fs;
use std::io;
use std::path::PathBuf;

use badm::commands;
use badm::paths;
use badm::{self, FileHandler};

use crate::common::{
    dotfiles_dir, home_dir, mock_config_file, mock_dotfile_in, stow_dir,
};

#[ignore]
#[test]
fn store_dotfiles_test() -> io::Result<()> {
    mock_config_file()?;

    let dotfile_path = mock_dotfile_in(home_dir())?;

    let expected_stow_path = stow_dir().join(dotfile_path.file_name().unwrap());

    let stow_path = commands::store_dotfile(&dotfile_path)?;

    assert!(expected_stow_path.exists());
    assert_eq!(expected_stow_path, stow_path);

    Ok(())
}

#[ignore]
#[test]
fn restore_dotfile_test() -> io::Result<()> {
    mock_config_file()?;

    // mock dotfile and corresponding symlink
    let dotfile_path = mock_dotfile_in(stow_dir())?;

    let stripped_dotfile_path = dotfile_path.strip_prefix(dotfiles_dir()).unwrap();

    let symlink_path = PathBuf::from("/").join(stripped_dotfile_path);
    fs::create_dir_all(symlink_path.parent().unwrap())?;
    FileHandler::create_symlink(&dotfile_path, &symlink_path)?;

    let actual_dst_path = commands::restore_dotfile(dotfile_path)?;

    assert!(!paths::is_symlink(&symlink_path));
    assert_eq!(actual_dst_path, symlink_path);

    Ok(())
}

#[ignore]
#[test]
fn deploy_dotfile_test() -> io::Result<()> {
    mock_config_file()?;

    // mock the stowed dotfile
    let dotfile_path = mock_dotfile_in(stow_dir())?;

    let stripped_dotfile_path = dotfile_path
        .strip_prefix(dotfiles_dir())
        .expect("Not able to strip prefix");

    assert!(dotfile_path.exists());

    let expected_symlink_path = PathBuf::from("/").join(stripped_dotfile_path);

    commands::deploy_dotfile(&dotfile_path, &expected_symlink_path)?;

    assert_eq!(fs::read_link(expected_symlink_path)?, dotfile_path);

    Ok(())
}
