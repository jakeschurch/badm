use std::fs;
use std::io::{self, Error, ErrorKind};
use std::path::{Path, PathBuf};

use crate::paths::{is_symlink, join_full_paths};
use crate::Config;
use crate::FileHandler;

pub fn store_dotfile(path: &Path) -> io::Result<PathBuf> {
    // create destination path
    let dots_dir = match Config::get_dots_dir() {
        Some(dir) => dir,
        None => {
            let err = io::Error::new(
                io::ErrorKind::NotFound,
                "Not able to complete operation because BADM_DIR was not set. Please \
                 run `badm set-dir=<DIR> first.`",
            );
            return Err(err);
        }
    };

    // TODO: substitute for normalize_path
    let path = if path.is_relative() {
        fs::canonicalize(path)?
    } else {
        path.to_path_buf()
    };

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
}

/// Dotfile is removed from the set dotfiles directory and moved to its symlink location.
/// The input can either be a dotfile's symlink path or the dotfile path itself.
pub fn restore_dotfile(path: PathBuf) -> io::Result<()> {
    // get src and dst paths
    let (src_path, dst_path): (PathBuf, PathBuf) = if is_symlink(&path)? {
        let src_path = fs::read_link(&path)?;
        fs::remove_file(&path)?;

        (src_path, path)
    } else {
        // TODO: lift this out to is_dotfile fn

        // check to see if file is in dotfiles dir
        let dots_dir = Config::get_dots_dir().unwrap();

        if !path.starts_with(&dots_dir) {
            // throw error
            let err = Error::new(
                ErrorKind::InvalidInput,
                "input path not located in dotfiles dir!",
            );
            return Err(err);
        };
        let dst_path =
            PathBuf::from("/").join(path.strip_prefix(dots_dir).unwrap().to_path_buf());

        if dst_path.exists() {
            fs::remove_file(&dst_path)?;
        };

        (path, dst_path)
    };

    FileHandler::move_file(&src_path, &dst_path)?;

    Ok(())
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
// REVIEW: not enough checks - ensure valid entry
pub fn deploy_dotfile(src: &Path, dotfiles_dir: &Path) -> io::Result<()> {
    let dst_symlink = PathBuf::from("/").join(
        src.strip_prefix(dotfiles_dir)
            .expect("Not able to create destination path"),
    );

    // if symlink already exists and points to src file, early return
    if dst_symlink.exists() && fs::read_link(&dst_symlink)? == src {
        return Ok(());
    };

    let dst_dir = dst_symlink.parent().unwrap();
    if !dst_dir.exists() {
        fs::create_dir_all(dst_dir)?;
    };

    FileHandler::create_symlink(&src, &dst_symlink)
}
