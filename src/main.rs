#![allow(clippy::all)]
use std::env;
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::prelude::*;
use std::io::{self, BufWriter};
use std::path::{Path, PathBuf};

/// Currently, the code sources the location of the dotfiles directory set by BADM
/// in the env variable "BADM_DIR"
const BADM_DIR_VAR: &str = "BADM_DIR";

fn main() {
    println!("Hello, world!");
}

fn stow_dotfile<K: AsRef<OsStr>>(src: K) -> io::Result<()> {
    let src = src.as_ref();
    let src_path = fs::canonicalize(src)?;

    // create destination path
    let dots_dir = match Config::get_dots_dir(BADM_DIR_VAR) {
        Some(dir) => dir,
        None => {
            let err = io::Error::new(io::ErrorKind::NotFound, "Not able to complete operation because BADM_DIR was not set. Please run `badm set-home=<DIR> first.`");
            return Err(err);
        }
    };

    let dst_path = Path::new(&dots_dir).join(&src_path);

    // create directory if not available
    if !dst_path.exists() {
        fs::create_dir_all(&dst_path)?
    }

    // move dotfile to dotfiles directory
    FileHandler::store_file(&src_path, &dst_path)?;

    Ok(())
}

fn rollout_dotfile_symlinks() -> io::Result<()> {
    // find dotfiles home
    // TODO: handle panic
    let dots_dir = Config::get_dots_dir(BADM_DIR_VAR).unwrap();

    let create_dotfiles_symlink = |src: PathBuf| -> io::Result<()> {
        let dst_symlink = src
            .strip_prefix(BADM_DIR_VAR)
            .expect("Not able to create destination path");

        fs::hard_link(&src, dst_symlink)?;

        Ok(())
    };

    // iterate through and create vector of filenames
    let entries = DirectoryScanner::new().get_entries(dots_dir.as_ref())?;

    // rollout each symlink
    for entry in entries.into_iter() {
        create_dotfiles_symlink(entry)?;
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

struct FileHandler {}

impl FileHandler {
    /// store a file in the dotfiles directory, create a symlink at the original source of the stowed file.
    pub fn store_file<K: AsRef<OsStr>>(src: K, dst: &Path) -> io::Result<()> {
        let src = src.as_ref();

        FileHandler::move_file(src, &dst)?;

        fs::hard_link(&dst, &src)?;

        Ok(())
    }

    fn move_file<K: AsRef<OsStr>>(src: K, dst: &Path) -> io::Result<()> {
        let src = src.as_ref();

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

struct Config {}

impl Config {
    fn get_dots_dir(var_name: &'static str) -> Option<String> {
        match env::var(var_name) {
            Ok(location) => Some(location),
            Err(_) => None,
        }
    }

    fn set_dots_dir<K: AsRef<OsStr>>(location: K) {
        env::set_var(BADM_DIR_VAR, location)
    }
}
