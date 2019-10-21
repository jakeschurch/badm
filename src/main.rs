#![allow(clippy::all)]

/// Currently, the code sources the location of the dotfiles directory set by BADM
/// in the env variable "BADM_DIR"
const BADM_DIR_VAR: &str = "BADM_DIR";

fn main() {
    println!("Hello, world!");
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
}
