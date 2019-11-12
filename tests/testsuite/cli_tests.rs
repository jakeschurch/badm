//! Contains integration tests for the badm crate

use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::Command;

use crate::common::{
    badm_config, dotfiles_dir, home_dir, mock_config_file, mock_dotfile_in, stow_dir,
};

use badm::paths;
use badm::{Config, FileHandler};

#[cfg(not(windows))]
const EXE_PATH: &str = "./target/debug/badm";

fn mock_command() -> Command {
    Command::new(EXE_PATH)
}

#[ignore]
#[test]
fn run_set_dir_test() -> io::Result<()> {
    if badm_config().exists() {
        fs::remove_file(badm_config())?;
    };

    mock_command()
        .arg("set-dir")
        .arg(dotfiles_dir())
        .output()
        .expect("failed to execute badm set-dir");

    let actual_dotfiles_dir = Config::get_dots_dir().unwrap();

    assert_eq!(actual_dotfiles_dir, dotfiles_dir());
    Ok(())
}

#[ignore]
#[test]
fn run_stow_test() -> io::Result<()> {
    mock_config_file()?;

    let file = mock_dotfile_in(home_dir()).expect("unable to mock input dotfile");

    let expected_stow_path = stow_dir().join(&file.file_name().unwrap());

    let output = mock_command()
        .arg("stow")
        .arg(&file)
        .output()
        .expect("failed to execute badm stow");

    assert!(output.status.success());
    assert_eq!(fs::read_link(file).unwrap(), expected_stow_path);
    assert!(expected_stow_path.exists());

    Ok(())
}

#[ignore]
#[test]
fn run_stow_multiple_test() -> io::Result<()> {
    mock_config_file()?;

    let mut input_path_vec: Vec<PathBuf> = vec![];

    for _ in 0..4 {
        let file = mock_dotfile_in(home_dir()).expect("unable to mock input dotfile");
        input_path_vec.push(file);
    }

    mock_command()
        .arg("stow")
        .args(&input_path_vec)
        .output()
        .expect("failed to execute badm stow");

    for file in input_path_vec.iter() {
        let expected_stow_path = stow_dir().join(file.file_name().unwrap());

        assert_eq!(fs::read_link(file).unwrap(), expected_stow_path);
        assert!(expected_stow_path.exists());
    }

    Ok(())
}

#[ignore]
#[test]
fn run_deploy_test() -> io::Result<()> {
    mock_config_file()?;

    let file =
        mock_dotfile_in(stow_dir().join(".config")).expect("failed to mock dotfile");

    let expected_deploy_path = home_dir().join(".config").join(file.file_name().unwrap());

    mock_command()
        .arg("deploy")
        .arg(&file)
        .output()
        .expect("failed to execute badm deploy");

    assert!(file.exists());
    assert_eq!(fs::read_link(expected_deploy_path).unwrap(), file);

    Ok(())
}

#[ignore]
#[test]
fn run_restore_dotfile_test() -> io::Result<()> {
    mock_config_file()?;

    let dotfile = mock_dotfile_in(stow_dir()).expect("failed to mock dotfile");

    let expected_restore_path = home_dir().join(&dotfile.file_name().unwrap());

    mock_command()
        .args(&["restore", dotfile.to_str().unwrap()])
        .output()
        .expect("failed to execute badm restore");

    assert!(expected_restore_path.exists());

    Ok(())
}

#[ignore]
#[test]
fn run_restore_symlink_test() -> io::Result<()> {
    mock_config_file()?;

    let dotfile = mock_dotfile_in(stow_dir()).expect("failed to mock dotfile");
    let expected_restore_path = home_dir().join(&dotfile.file_name().unwrap());

    FileHandler::create_symlink(&dotfile, &expected_restore_path)?;

    mock_command()
        .args(&["restore", expected_restore_path.to_str().unwrap()])
        .output()
        .expect("failed to execute badm restore");

    assert!(!paths::is_symlink(&expected_restore_path));

    Ok(())
}
