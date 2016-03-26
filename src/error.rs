use std::{error, fmt};

// Potential errors that can happen while constructing a router.
#[derive(Debug)]
pub enum RouterError {
    TooFewRoutes,
    BadSet
}

impl error::Error for RouterError {
    fn description(&self) -> &str {
        match *self {
            RouterError::TooFewRoutes => { "Less than 2 routes provided to the set" },
            RouterError::BadSet => { "Error making RegexSet" }
        }
    }
}

impl fmt::Display for RouterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RouterError::TooFewRoutes => {
                write!(f, "Cannot make RegexSet with fewer than 2 routes.")
            },
            RouterError::BadSet => {
                write!(f, "Error making RegexSet.")
            }
        }
    }
}
