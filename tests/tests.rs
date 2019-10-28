extern crate badm_core;
use badm_core::{
    create_dotfiles_symlink, is_symlink, join_full_paths, stow_dotfile, unstow_dotfile,
    FileHandler,
};

extern crate dirs;
use dirs::home_dir;

use std::env::{self, var};
use std::fs::{self, File};
use std::io;
use std::io::prelude::*;
use std::path::PathBuf;

const BADM_TEST_DIR_VAR: &str = "BADM_TEST_DIR";

fn dotfiles_dir() -> PathBuf {
    home_dir().unwrap().join(".dotfiles")
}

fn mock_dotfiles_dir() -> io::Result<()> {
    let dots_dir = dotfiles_dir();

    if !dots_dir.exists() {
        fs::create_dir_all(&dots_dir)?;
    }
    env::set_var(BADM_TEST_DIR_VAR, dots_dir);

    assert_eq!(
        PathBuf::from(var(BADM_TEST_DIR_VAR).unwrap()),
        dotfiles_dir()
    );

    Ok(())
}

fn create_input_dotfile(dotfile_path: &PathBuf) -> io::Result<()> {
    // ensure parent dir exists
    let parent_dir = dotfile_path.parent().unwrap();
    if !parent_dir.exists() {
        fs::create_dir_all(parent_dir)?;
    };

    if !dotfile_path.exists() {
        // create .profile dotfile
        let mut dotfile = File::create(&dotfile_path)?;
        dotfile.write_all(b"alias la=\"ls -la\"")?;
        dotfile.sync_all().unwrap();
    }

    Ok(())
}

#[ignore]
#[test]
fn unstow_dotfile_test() -> io::Result<()> {
    mock_dotfiles_dir()?;

    // mock dotfile and corresponding symlink
    let dotfile_path = join_full_paths(dotfiles_dir(), home_dir().unwrap())
        .unwrap()
        .join(".fishrc");
    create_input_dotfile(&dotfile_path)?;

    let symlink_path = home_dir().unwrap().join(".fishrc");

    if !symlink_path.exists() {
        FileHandler::create_symlink(&dotfile_path, &symlink_path)?;
    };

    unstow_dotfile(&dotfile_path, BADM_TEST_DIR_VAR)?;

    assert!(!is_symlink(symlink_path)?);

    Ok(())
}

#[ignore]
#[test]
fn stow_dotfiles_test() -> io::Result<()> {
    mock_dotfiles_dir()?;

    let dotfile_path = home_dir().unwrap().join(".bash_profile");

    create_input_dotfile(&dotfile_path)?;

    let expected_stow_path = join_full_paths(dotfiles_dir(), home_dir().unwrap())
        .unwrap()
        .join(".bash_profile");

    let stow_path = stow_dotfile(&dotfile_path, BADM_TEST_DIR_VAR)?;

    assert_eq!(fs::read_link(dotfile_path)?, stow_path);
    assert_eq!(expected_stow_path, stow_path);

    Ok(())
}

#[ignore]
#[test]
fn create_dotfiles_symlink_test() -> io::Result<()> {
    mock_dotfiles_dir()?;

    // mock the stowed dotfile
    let stowed_dotfile = join_full_paths(dotfiles_dir(), home_dir().unwrap())
        .unwrap()
        .join(".config/.profile");

    create_input_dotfile(&stowed_dotfile)?;

    let expected_symlink_path = home_dir().unwrap().join(".config/.profile");

    create_dotfiles_symlink(&stowed_dotfile, BADM_TEST_DIR_VAR)?;

    assert_eq!(fs::read_link(expected_symlink_path)?, stowed_dotfile);

    Ok(())
}
