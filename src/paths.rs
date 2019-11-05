use std::fs::{self, File};
use std::io;
use std::io::prelude::*;
use std::path::{Path, PathBuf, StripPrefixError};

pub fn is_symlink(path: &Path) -> io::Result<bool> {
    fs::symlink_metadata(path).and_then(|md| Ok(md.file_type().is_symlink()))
}

pub fn sanitize_path(path: &mut Path) -> io::Result<PathBuf> {
    if path.is_relative() {
        fs::canonicalize(path)
    } else {
        Ok(path.to_path_buf())
    }
}

pub(crate) fn read_path(path: &Path) -> io::Result<String> {
    let mut file = File::open(path)?;
    read_file(&mut file)
}

pub(crate) fn read_file(file: &mut File) -> io::Result<String> {
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

/// Joins two full paths together.
/// If path is unix and second path argument contains root directory, it is stripped.
///
/// This behavior is an anti-use case of [`PathBuf::join`], but is valid for the need to
/// replicate directory paths containing root within others.
///
/// [`PathBuf`::join]: struct.PathBuf.html#method.join
///
/// # Examples
///
/// ```
/// use badm_core::paths::join_full_paths;
/// use std::path::PathBuf;
/// # use std::path;
///
/// let path_1 = PathBuf::from("/home/ferris/.dotfiles");
/// let path_2 = PathBuf::from("/home/ferris");
///
/// assert_eq!(
///     join_full_paths(&path_1, &path_2),
///     Ok(PathBuf::from("/home/ferris/.dotfiles/home/ferris"))
/// );
/// ```
pub fn join_full_paths(
    path_1: &Path,
    path_2: &Path,
) -> Result<PathBuf, StripPrefixError> {
    if path_2.has_root() && cfg!(target_family = "unix") {
        let path_2 = path_2.strip_prefix("/")?;
        return Ok(path_1.join(path_2));
    };
    Ok(path_1.join(path_2))
}
