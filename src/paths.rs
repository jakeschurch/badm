use std::fs::{self, File};
use std::io;
use std::io::prelude::*;
use std::path::{Path, PathBuf, StripPrefixError};

/// Wrapper for is_symlink for paths
pub fn is_symlink(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .map(|md| md.file_type().is_symlink())
        .unwrap_or(false)
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
/// [`PathBuf::join`]: std/path/struct.PathBuf.html#method.join
///
/// # Examples
///
/// ```
/// use badm::paths::join_full_paths;
/// use std::path::PathBuf;
///
/// assert_eq!(
///     join_full_paths(
///         &PathBuf::from("/home/ferris/.dotfiles"),
///         &PathBuf::from("/home/ferris")
///     ),
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile;

    #[test]
    fn is_symlink_test() -> io::Result<()> {
        // mock files
        let builder = tempfile::Builder::new();

        let dir = builder.tempdir()?.into_path();
        let (_, file) = builder.tempfile_in(&dir)?.keep()?;
        let symlink_dst = dir.join("symlink_dst");

        crate::FileHandler::create_symlink(&file, &symlink_dst)?;

        assert!(symlink_dst.exists());

        assert!(is_symlink(&symlink_dst));

        Ok(())
    }
}
