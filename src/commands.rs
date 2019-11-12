//! Includes the commands used by the badm crate/application.

use std::fs;
use std::io::{self, Error, ErrorKind};
use std::path::{Path, PathBuf};

use crate::paths::{is_symlink, join_full_paths};
use crate::Config;
use crate::FileHandler;

/// Take input from file at path and store in set dotfiles directory.
pub fn store_dotfile(path: &Path) -> io::Result<PathBuf> {
    if let Some(dots_dir) = Config::get_dots_dir() {
        // create destination path
        let dst_path = join_full_paths(&dots_dir, &path).unwrap();

        // if symlink already exists and points to src file, early return
        if dst_path.exists() && fs::read_link(&dst_path)? == path {
            return Ok(dst_path);
        };

        // create directory if not available
        let dst_dir = dst_path.parent().unwrap();

        if !dst_dir.exists() {
            fs::create_dir_all(dst_dir)?;
        };

        // move dotfile to dotfiles directory
        FileHandler::move_file(&path, &dst_path)?;

        Ok(dst_path)
    } else {
        let err = io::Error::new(
            io::ErrorKind::NotFound,
            "Not able to complete operation because BADM_DIR was not set. Please run \
             `badm set-dir=<DIR> first.`",
        );
        Err(err)
    }
}

/// Dotfile is removed from set dotfiles directory and moved to its symlink location.
/// The input can either be a dotfile's symlink path or the path of the dotfile itself.
///
/// Returns destination path.
pub fn restore_dotfile(path: PathBuf) -> io::Result<PathBuf> {
    // get src and dst paths
    let (src_path, dst_path): (PathBuf, PathBuf) = if is_symlink(&path) {
        (fs::read_link(&path)?, path)
    } else {
        let restore_path = path.strip_prefix(Config::get_dots_dir().unwrap()).unwrap();
        let dst_path = PathBuf::from("/").join(restore_path);

        (path, dst_path)
    };

    let is_dotfile = |path: &Path| -> bool {
        crate::Config::get_dots_dir()
            .map(|dir| path.starts_with(dir))
            .unwrap()
    };

    // check to see if src path exists in dotfiles directory, if not: it is invalid input
    if !is_dotfile(&src_path) {
        // throw error
        let err = Error::new(
            ErrorKind::InvalidInput,
            "input path not located in dotfiles dir!",
        );
        return Err(err);
    };

    if dst_path.exists() {
        fs::remove_file(&dst_path)?;
    };

    FileHandler::move_file(&src_path, &dst_path)?;

    Ok(dst_path)
}

/// Create symlinks in directories relative to the dotfiles' directory hierarchy
/// for deploying new configurations.
/// Example: if Ferris downloaded a git dotfiles repo onto a new machine into the
/// .dotfiles directory:
///
/// <pre>
/// /home
/// └── ferris
///     └── .dotfiles
///         └── home
///             └── ferris
///                 └── .config
///                     └── .gitconfig
/// </pre>
///
/// They could easily setup their configuration files on this machine by setting
/// up the relative symlinks by storing their configuration files in one directory, and
/// have that directory mimic the directory hiearchy of the target machine. This is what
/// BADM hopes to achieve.
///
/// <pre>
/// /home
/// └── ferris
///     ├── .config
///     │   └── .gitconfig -> /home/ferris/.dotfiles/home/ferris/.config/.gitconfig
///     └── .dotfiles
///         └── home
///             └── ferris
///                 └── .config
///                     └── .gitconfig
/// </pre>
///
/// Directories to replicate the stored dotfile's directory structure will be created if
/// not found.
// REVIEW: not enough checks - need to ensure valid entry.
pub fn deploy_dotfile(src: &Path, dst: &Path) -> io::Result<()> {
    // if symlink already exists and points to src file, early return
    if dst.exists() && fs::read_link(&dst)? == src {
        return Ok(());
    };

    let dst_dir = dst.parent().unwrap();
    if !dst_dir.exists() {
        fs::create_dir_all(dst_dir)?;
    };

    FileHandler::create_symlink(&src, &dst)
}
