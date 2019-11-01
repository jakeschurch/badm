//! badm is a command-line tool use to store dotfiles, or configuration files.

#![allow(dead_code)]

pub mod commands;
pub(crate) mod config;
pub mod paths;

pub use crate::config::Config;

use std::fs::{self, File};
use std::io::{self, prelude::*, BufWriter};
use std::path::{Path, PathBuf};

// TODO: create dotfile struct

pub struct DirectoryScanner {
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
            .map(fs::canonicalize)
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
                    self.entries.push(path)
                }
            }
        }
        Ok(())
    }
}

pub struct FileHandler;

impl FileHandler {
    /// Store a file in the dotfiles directory, create a symlink at the original
    /// source of the stowed file.
    pub fn store_file(src: &Path, dst: &Path) -> io::Result<()> {
        FileHandler::move_file(src, dst)?;

        FileHandler::create_symlink(dst, src)?;

        Ok(())
    }

    /// Create a symlink at "dst" pointing to "src."
    ///
    /// For Unix platforms, std::os::unix::fs::symlink is used to create
    /// symlinks. For Windows, std::os::windows::fs::symlink_file is used.
    // TODO|BUGFIX: ensure or throw error when dst parent does not exist
    pub fn create_symlink(src: &Path, dst: &Path) -> io::Result<()> {
        #[cfg(not(target_os = "windows"))]
        use std::os::unix::fs::symlink;

        #[cfg(target_os = "windows")]
        use std::os::windows::fs::symlink_file as symlink;
        symlink(src, dst)?;

        Ok(())
    }

    pub fn move_file(src: &Path, dst: &Path) -> io::Result<()> {
        // read file to String
        let mut contents = String::new();
        let mut f = File::open(src)?;
        f.read_to_string(&mut contents)?;

        // write String contents to dst file
        let dst_file = File::create(dst)?;
        let mut writer = BufWriter::new(dst_file);
        writer.write_all(contents.as_bytes())?;

        // remove file at src location
        fs::remove_file(&src)?;

        Ok(())
    }
}
