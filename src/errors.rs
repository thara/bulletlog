use std::fmt;

#[derive(Debug, Clone)]
pub struct UnsupportedError;

impl fmt::Display for UnsupportedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unsupported")
    }
}

impl std::error::Error for UnsupportedError {}
