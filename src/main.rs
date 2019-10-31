// TODO: create integration tests for main

#![allow(clippy::all)]
// TEMP: since in large dev production
#![allow(dead_code)]

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[macro_use]
extern crate clap;

use clap::{App, Arg, Values};

use badm_core::paths::{is_symlink, normalize_path};
use badm_core::{create_dotfile_symlink, stow_dotfile, Config};

fn main() -> io::Result<()> {
    // TODO
    // let unstow_subcommand = App::new("unstow");
    // let deploy_subcommand = App::new("deploy");

    let set_dir_subcommand = App::new("set-dir")
        .about("set path of dotfiles directory")
        .version("1.0")
        .display_order(1)
        .arg(
            Arg::with_name("directory")
                .help("directory to store dotfiles")
                .required(true),
        );

    let stow_subcommand = App::new("stow")
        .about("store input files in the dotfiles directory, and replace the file's original path with a symlink")
        .version("0.1")
        .display_order(2)
        .arg(
            Arg::with_name("files")
                .help("path of the file/files to be stored in the dotfiles directory")
                .required(true)
                .multiple(true),
        );

    let matches = App::new("badm")
        .about(crate_description!())
        .version(crate_version!())
        .author(crate_authors!())
        .after_help("https://github.com/jakeschurch/badm")
        .subcommands(vec![set_dir_subcommand, stow_subcommand])
        .get_matches();

    match matches.subcommand() {
        ("set-dir", Some(set_dir_matches)) => {
            let dir_path = set_dir_matches.value_of("directory").unwrap();
            set_dir(dir_path)?
        }
        ("stow", Some(stow_matches)) => {
            let input_paths = stow_matches.values_of("files").unwrap();
            stow(input_paths)?
        }
        _ => unreachable!(),
    }
    Ok(())
}

fn set_dir<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let _ = Config::set_dots_dir(path.as_ref())?;
    Ok(())
}

fn stow(values: Values) -> io::Result<()> {
    for value in values.into_iter() {
        let path = PathBuf::from(value);

        let path = normalize_path(&path)?;

        // TODO: push down is symlink and return error
        if path.is_file() && !is_symlink(&path)? {
            stow_dotfile(&path)?;
        };
    }
    Ok(())
}

fn deploy_dotfile_symlinks() -> io::Result<()> {
    // find dotfiles dir
    // TODO: Introduce custom errors
    let dots_dir = Config::get_dots_dir().unwrap();

    // iterate through and create vector of filenames
    let entries = DirectoryScanner::new().get_entries(dots_dir.as_ref())?;

    // deploy each symlink
    for entry in entries.into_iter() {
        create_dotfile_symlink(&entry)?;
    }

    Ok(())
}

struct DirectoryScanner {
    entries: Vec<PathBuf>,
}

impl DirectoryScanner {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn get_entries(&mut self, dir: &Path) -> io::Result<(Vec<PathBuf>)> {
        self.collect_entries(dir)?;

        self.entries = self
            .entries
            .iter_mut()
            .map(|entry| fs::canonicalize(entry))
            .filter_map(Result::ok)
            .collect::<Vec<PathBuf>>();

        Ok(self.entries.clone())
    }

    fn collect_entries(&mut self, dir: &Path) -> io::Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    if !path.ends_with(".git") {
                        self.collect_entries(&path)?;
                    }
                } else {
                    self.entries.push(path.into())
                }
            }
        }
        Ok(())
    }
}
