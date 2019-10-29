#![allow(clippy::all)]
// TEMP: since in large dev production
#![allow(dead_code)]

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use badm_core::config::{Config, BADM_DIR_VAR};
use badm_core::create_dotfiles_symlink;

fn main() {
    println!("Hello, world!");
}

fn rollout_dotfile_symlinks() -> io::Result<()> {
    // find dotfiles home
    let dots_dir = Config::get_dots_dir(BADM_DIR_VAR).unwrap();

    // iterate through and create vector of filenames
    let entries = DirectoryScanner::new().get_entries(dots_dir.as_ref())?;

    // rollout each symlink
    for entry in entries.into_iter() {
        create_dotfiles_symlink(&entry, BADM_DIR_VAR)?;
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
