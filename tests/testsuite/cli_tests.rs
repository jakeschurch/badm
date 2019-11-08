//! Contains integration tests for the badm crate

use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::Command;

use crate::common::{mock_dotfile_in, BADM_CONFIG, DOTFILES_DIR, HOME_DIR};
use crate::core_tests::STOW_DIR;

use badm_core::Config;

#[cfg(not(windows))]
const EXE_PATH: &str = "./target/debug/badm";

fn mock_command() -> Command {
    Command::new(EXE_PATH)
}

#[ignore]
#[test]
fn run_set_dir_test() -> io::Result<()> {
    if BADM_CONFIG.exists() {
        fs::remove_file(BADM_CONFIG.to_path_buf())?;
    };

    mock_command()
        .arg("set-dir")
        .arg(DOTFILES_DIR.to_path_buf())
        .output()
        .expect("failed to execute badm set-dir");

    let actual_dotfiles_dir = Config::get_dots_dir().unwrap();

    assert_eq!(actual_dotfiles_dir, DOTFILES_DIR.to_path_buf());
    Ok(())
}

#[ignore]
#[test]
fn run_stow_test() {
    let file =
        mock_dotfile_in(HOME_DIR.to_path_buf()).expect("unable to mock input dotfile");

    let expected_stow_path = STOW_DIR.to_path_buf().join(&file.file_name().unwrap());

    mock_command()
        .arg("stow")
        .arg(&file)
        .output()
        .expect("failed to execute badm stow");

    assert_eq!(fs::read_link(file).unwrap(), expected_stow_path);
    assert!(expected_stow_path.exists());
}

#[ignore]
#[test]
fn run_stow_multiple_test() {
    let mut input_path_vec: Vec<PathBuf> = vec![];

    for _ in 0..4 {
        let file = mock_dotfile_in(HOME_DIR.to_path_buf())
            .expect("unable to mock input dotfile");
        input_path_vec.push(file);
    }

    let glob = HOME_DIR.to_path_buf().join("*");

    mock_command()
        .arg("stow")
        .arg(&glob)
        .output()
        .expect("failed to execute badm stow");

    for file in input_path_vec.iter() {
        let expected_stow_path = STOW_DIR.to_path_buf().join(file.file_name().unwrap());

        assert_eq!(fs::read_link(file).unwrap(), expected_stow_path);
        assert!(expected_stow_path.exists());
    }
}
// #[ignore]
// #[test]
// fn run_deploy_test() {
//     mock_command().arg("deploy");
// }
//
// #[ignore]
// #[test]
// fn run_restore_test() {
//     mock_command().arg("restore");
// }
