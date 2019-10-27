#![allow(clippy::all)]

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use badm_core::config::{Config, BADM_DIR_VAR};
use badm_core::join_full_paths;

fn main() {
    println!("Hello, world!");
}

// REVIEW:
//     - recursive flag?
//     - glob patterns?
// TEMP: env_var input argument will go away when we convert to toml config
fn stow_dotfile(env_var: &'static str, src: &PathBuf) -> io::Result<()> {
    // create destination path
    let dots_dir = match Config::get_dots_dir(env_var) {
        Some(dir) => dir,
        None => {
            let err = io::Error::new(io::ErrorKind::NotFound, "Not able to complete operation because BADM_DIR was not set. Please run `badm set-home=<DIR> first.`");
            return Err(err);
        }
    };

    let dst_path = join_full_paths(&PathBuf::from(dots_dir), &src).unwrap();

    // create directory if not available
    if !dst_path.exists() {
        let dst_dir = dst_path.parent().unwrap();

        fs::create_dir_all(dst_dir)?;
    }

    // move dotfile to dotfiles directory
    FileHandler::store_file(&src, &dst_path)?;

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

        FileHandler::create_symlink(&src, dst_symlink)?;

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

/// Create symlinks in directories relative to the dotfiles' directory hierarchy
/// for "rolling out" new configurations.
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
fn create_dotfiles_symlink(src: &PathBuf, env_var: &'static str) -> io::Result<()> {
    let dots_dir = Config::get_dots_dir(env_var).unwrap();
    let dst_symlink = PathBuf::from("/").join(
        src.strip_prefix(dots_dir)
            .expect("Not able to create destination path"),
    );

    // if symlink already exists and points to src file, early return
    if dst_symlink.exists() && fs::read_link(&dst_symlink)? == *src {
        println!("Destination file link exists");
        return Ok(());
    };

    let dst_dir = dst_symlink.parent().unwrap();
    if !dst_dir.exists() {
        fs::create_dir_all(dst_dir)?;
    };

    FileHandler::create_symlink(&src, &dst_symlink)
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

// REVIEW: Turn FileHandler into a trait?
struct FileHandler {}

impl FileHandler {
    /// store a file in the dotfiles directory, create a symlink at the original source of the stowed file.
    pub fn store_file(src: &Path, dst: &Path) -> io::Result<()> {
        FileHandler::move_file(&src, &dst)?;

        FileHandler::create_symlink(dst, src)?;

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

    fn create_symlink(src: &Path, dst: &Path) -> io::Result<()> {
        symlink(src, dst)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    const BADM_TEST_DIR_VAR: &str = "BADM_TEST_DIR";

    fn home_dir() -> PathBuf {
        PathBuf::from("/tmp/badm/home/ferris")
    }

    fn mock() -> io::Result<()> {
        let dots_dir = home_dir().join(".dotfiles");

        if let Err(_) = fs::metadata(&dots_dir) {
            fs::create_dir_all(&dots_dir)?;
        }
        env::set_var(BADM_TEST_DIR_VAR, dots_dir);

        assert_eq!(
            env::var(BADM_TEST_DIR_VAR),
            Ok("/tmp/badm/home/ferris/.dotfiles".to_owned())
        );

        Ok(())
    }

    fn teardown() -> io::Result<()> {
        let home_dir = home_dir();
        let temp_dir = home_dir.parent().unwrap();

        fs::remove_dir_all(temp_dir)?;
        Ok(())
    }

    #[test]
    fn stow_dotfiles_test() -> io::Result<()> {
        mock()?;

        let dotfile_location = home_dir().join(".profile");
        let mut dotfile = File::create(&dotfile_location)?;
        dotfile.write_all(b"alias la=\"ls -la\"")?;

        stow_dotfile(BADM_TEST_DIR_VAR, &dotfile_location)?;

        assert!(fs::symlink_metadata(dotfile_location)?.is_file() == false);
        teardown()?;
        Ok(())
    }
}
