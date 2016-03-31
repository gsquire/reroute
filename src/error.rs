use std::{error, fmt};

// Potential errors that can happen while constructing a router.
#[derive(Debug, PartialEq)]
pub enum RouterError {
    TooFewRoutes,
    BadSet
}

impl error::Error for RouterError {
    fn description(&self) -> &str {
        match *self {
            RouterError::TooFewRoutes => { "No routes provided for the router" }
            RouterError::BadSet => { "Error making RegexSet" }
        }
    }
}

impl fmt::Display for RouterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RouterError::TooFewRoutes => {
                write!(f, "Cannot make a router with zero routes.")
            },
            RouterError::BadSet => {
                write!(f, "Error making RegexSet.")
            }
        }
    }
}
