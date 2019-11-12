use std::io;

#[allow(missing_docs)]
#[derive(Debug, Fail)]
pub enum InputError {
    #[fail(display = "Could not parse toml: {:?}", err)]
    InvalidToml { err: toml::de::Error },
    #[fail(display = "Could not parse input: {:?}: ", err)]
    BadInput { err: io::Error },
}

impl From<io::Error> for InputError {
    fn from(err: io::Error) -> Self {
        Self::BadInput { err }
    }
}

impl From<toml::de::Error> for InputError {
    fn from(err: toml::de::Error) -> Self {
        Self::InvalidToml { err }
    }
}
