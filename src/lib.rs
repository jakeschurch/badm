//! `badm` is a tool that stores your configuration files, or
//! [dotfiles](https://en.wikipedia.org/wiki/Hidden_file_and_hidden_directory), in a directory that replicates the directory hierarchy of the
//! dotfiles' original path, and creates symlinks to their original paths. This creates a
//! standardized and systematic approach for managing, deploying, and sharing dotfiles
//! among different systems and users.
//!
//! badm is ultimately "But Another Dotfiles Manager".
//!
//! # Examples
//!
//! - ferris has created a directory to store their dotfiles at `~/.dots`
//! - `badm set-dir ~/.dots` sets the BADM dotfiles dir at `~/.dots`
//! - badm will search for a badm config file at one of the two valid locations: `$HOME`
//!   and `$XDG_CONFIG_HOME`. If the config file not found, badm will create it under
//!   `$HOME`
//!
//! <pre>
//! /home
//! └── ferris
//!     └── .dots
//!         ├── .badm.toml
//!         └── .gitconfig
//! </pre>
//!
//!
//! - to store `~/.gitconfig` as a dotfile, ferris runs `badm stow ~/.gitconfig`
//!   _(relative paths work as well)_
//! - badm replicates the path of the dotfile under the `~/.dots` directory
//! - the dotfile is moved to this new path in the set dotfiles directory and symlinked at
//!   its original path which points to its new path
//!
//! <pre>
//! /home
//! └── ferris
//!     ├── .badm.toml
//!     ├── .dots
//!     │   └── home
//!     │       └── ferris
//!     │           └── .gitconfig
//!     └── .gitconfig -> /home/ferris/.dots/home/ferris/.gitconfig
//! </pre>
//!
//! # Commands
//!
//! - `badm set-dir <DIRECTORY>` - set dotfiles directory location, if the location is not
//!   created BADM has the ability to create one for you
//! - `badm stow <FILE>` - store a file in the dotfiles directory, create a symlink at the
//!   original source of the stowed file.
//! - `badm deploy <FILE>` - for new configurations, create symlinks in directories
//!   relative to the dotfile's directory hierarchy. Directories to replicate the stored
//!   dotfile's directory structure will be created if not found.
//! - `badm restore <FILE>` - restore the stored file from the dotfiles directory and
//!   replace the symlink with the original file

#![allow(clippy::all)]
// #![deny(missing_docs)]
#![allow(dead_code)]

pub mod commands;
pub(crate) mod config;
mod errors;
pub mod paths;

pub use crate::config::Config;
pub use crate::errors::InputError;

#[macro_use] extern crate failure;

use std::fs::{self, File};
use std::io::{self, prelude::*, BufWriter};
use std::path::{Path, PathBuf};

// TODO: create dotfile struct

/// Struct used to traverse directories and collect entries located within.
pub struct DirScanner {
    entries: Vec<PathBuf>,
    recursive: bool,
}

impl DirScanner {
    /// Given a directory, traverse path and get entries located within `dir`.
    /// If the [`DirScanner::recursive`] method is not called before get_entries, it will
    /// only traverse one level below.
    ///
    /// [`DirScanner::recursive`]: struct.DirScanner.html/#method.recursive
    pub fn get_entries(mut self, dir: &Path) -> io::Result<Vec<PathBuf>> {
        self.collect_entries(dir)?;

        self.entries = self
            .entries
            .into_iter()
            .map(|path| {
                if path.is_relative() {
                    fs::canonicalize(path)
                } else {
                    Ok(path)
                }
            })
            .filter_map(Result::ok)
            .collect();

        Ok(self.entries)
    }

    fn new() -> Self {
        DirScanner {
            entries: Vec::new(),
            recursive: false,
        }
    }

    /// Builder method to set recursive flag to `true` when scanning directory.
    pub fn recursive(mut self) -> Self {
        self.recursive = true;
        self
    }

    fn collect_entries(&mut self, dir: &Path) -> io::Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let path = entry.map(|e| e.path())?;

                if path.is_dir() && self.recursive {
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

impl Default for DirScanner {
    fn default() -> Self {
        DirScanner {
            entries: vec![],
            recursive: false,
        }
    }
}

/// Moves, stores, and creates files and symlinks.
pub struct FileHandler;

impl FileHandler {
    /// Store a file in the dotfiles directory, create a symlink at the original
    /// source of the stowed file.
    pub fn store_file(src: &Path, dst: &Path) -> io::Result<()> {
        FileHandler::move_file(src, dst)?;
        FileHandler::create_symlink(dst, src)
    }

    /// Read file at path src and write to created/truncated file at path dst.
    pub fn move_file(src: &Path, dst: &Path) -> io::Result<()> {
        // read file path to String
        let contents = crate::paths::read_path(src)?;

        // write String contents to dst file
        let dst_file = File::create(dst)?;
        let mut writer = BufWriter::new(dst_file);
        writer.write_all(contents.as_bytes())?;

        // remove file at src location
        fs::remove_file(&src)
    }

    /// Create a symlink at "dst" pointing to "src."
    ///
    /// For Unix platforms, [`std::os::unix::fs::symlink`] is used to create
    /// symlinks. For Windows, [`std::os::windows::fs::symlink_file`] is used.
    ///
    /// [`std::os::unix::fs::symlink`]: std/os/unix/fs/fn.symlink.html
    /// [`std::os::windows::fs::symlink_file`]: std/os/windows/fs/fn.symlink_file.html
    pub fn create_symlink(src: &Path, dst: &Path) -> io::Result<()> {
        #[cfg(not(target_os = "windows"))] use std::os::unix::fs::symlink;

        #[cfg(target_os = "windows")]
        use std::os::windows::fs::symlink_file as symlink;
        symlink(src, dst)
    }
}
