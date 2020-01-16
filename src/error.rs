use std::fmt;

// Potential errors that can happen while constructing a router.
#[derive(Debug)]
pub enum Error {
    BadRegex(::regex::Error),
}

impl From<::regex::Error> for Error {
    fn from(error: ::regex::Error) -> Error {
        Error::BadRegex(error)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::BadRegex(ref error) => write!(f, "{}", error),
        }
    }
}
