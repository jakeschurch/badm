pub mod config;

use std::fs::{self, File};
use std::io::{self, prelude::*, BufWriter};
use std::path::{self, Path, PathBuf};

pub struct FileHandler;

impl FileHandler {
    /// Store a file in the dotfiles directory, create a symlink at the original
    /// source of the stowed file.
    pub fn store_file(src: &Path, dst: &Path) -> io::Result<()> {
        FileHandler::move_file(&src, &dst)?;

        FileHandler::create_symlink(dst, src)?;

        Ok(())
    }

    /// Create a symlink at "dst" pointing to "src."
    ///
    /// For Unix platforms, std::os::unix::fs::symlink is used to create
    /// symlinks. For Windows, std::os::windows::fs::symlink_file is used.
    pub fn create_symlink(src: &Path, dst: &Path) -> io::Result<()> {
        #[cfg(not(target_os = "windows"))]
        use std::os::unix::fs::symlink;

        #[cfg(target_os = "windows")]
        use std::os::windows::fs::symlink_file as symlink;
        symlink(src, dst)?;
        Ok(())
    }

    fn move_file(src: &Path, dst: &Path) -> io::Result<()> {
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
/// use badm_core::join_full_paths;
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
// TODO: test windows root paths
pub fn join_full_paths(
    path_1: &PathBuf,
    path_2: &PathBuf,
) -> Result<PathBuf, path::StripPrefixError> {
    if path_2.has_root() && cfg!(target_family = "unix") {
        let path_2 = path_2.strip_prefix("/")?;
        return Ok(path_1.join(path_2));
    };
    Ok(path_1.join(path_2))
}
