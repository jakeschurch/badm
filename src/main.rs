#![allow(clippy::all)]
use std::env;
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::prelude::*;
use std::io::{self, BufWriter};
use std::path::Path;

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

struct FileHandler {}

impl FileHandler {
    // store a file in the dotfiles directory, create a symlink at the original source of the stowed file.
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
    fn get_dots_dir() -> Option<String> {
        match env::var(BADM_DIR_VAR) {
            Ok(location) => Some(location),
            Err(_) => None,
        }
    }

    fn set_dots_dir<K: AsRef<OsStr>>(location: K) {
        env::set_var(BADM_DIR_VAR, location)
    }
}
