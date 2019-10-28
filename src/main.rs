#![allow(clippy::all)]

use std::fs;
use std::io::{self, Error, ErrorKind};
use std::path::{Path, PathBuf};

use badm_core::config::{Config, BADM_DIR_VAR};
use badm_core::{create_dotfiles_symlink, is_symlink, FileHandler};

fn main() {
    println!("Hello, world!");
}

fn rollout_dotfile_symlinks() -> io::Result<()> {
    // find dotfiles home
    // TODO(#1): handle panic
    let dots_dir = Config::get_dots_dir(BADM_DIR_VAR).unwrap();

    // iterate through and create vector of filenames
    let entries = DirectoryScanner::new().get_entries(dots_dir.as_ref())?;

    // rollout each symlink
    for entry in entries.into_iter() {
        create_dotfiles_symlink(&entry, BADM_DIR_VAR)?;
    }

    Ok(())
}

/// Dotfile is removed from the set dotfiles directory and moved to its symlink location.
/// The input can either be a dotfile's symlink path or the dotfile path itself.
fn unstow_dotfile<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let path = path.as_ref().to_path_buf();

    // get src and dst paths
    let (src_path, dst_path): (PathBuf, PathBuf) = if is_symlink(&path)? {
        let src_path = fs::read_link(&path)?;
        fs::remove_file(&path)?;

        (src_path, path)
    } else {
        // check to see if file is in dotfiles dir
        let dots_dir = Config::get_dots_dir(BADM_DIR_VAR).unwrap();

        if !path.starts_with(&dots_dir) {
            // throw error
            let err = Error::new(
                ErrorKind::InvalidInput,
                "input path not located in dotfiles dir!",
            );
            return Err(err);
        };
        let dst_path = path.strip_prefix(dots_dir).unwrap().to_path_buf();

        (path, dst_path)
    };

    FileHandler::move_file(src_path, dst_path)?;

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

