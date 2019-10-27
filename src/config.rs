use std::env;
use std::ffi::OsStr;

/// TEMP: Currently, the code sources the location of the dotfiles directory set by BADM
/// in the env variable "BADM_DIR." This will be replaced in favor of a TOML configuration
/// file in a future release.
pub const BADM_DIR_VAR: &str = "BADM_DIR";

pub struct Config;

impl Config {
    pub fn get_dots_dir(var_name: &'static str) -> Option<String> {
        match env::var(var_name) {
            Ok(location) => Some(location),
            Err(_) => None,
        }
    }

    pub fn set_dots_dir<K: AsRef<OsStr>>(location: K) {
        env::set_var(BADM_DIR_VAR, location)
    }
}

