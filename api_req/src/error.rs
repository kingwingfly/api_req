//! Error module

use core::error::Error;
use core::fmt;

/// Error type for API
#[derive(Debug)]
pub enum ApiErr {
    /// Reqwest error
    Reqwest(reqwest::Error),
    /// Serde error; Contains the returned body
    Serde(String),
    /// Other error
    Other(String),
}

impl From<reqwest::Error> for ApiErr {
    fn from(e: reqwest::Error) -> Self {
        Self::Reqwest(e)
    }
}

impl fmt::Display for ApiErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Reqwest(e) => write!(f, "Reqwest error: {}", e),
            Self::Serde(e) => write!(f, "Serde error: {}", e),
            ApiErr::Other(e) => write!(f, "Other error: {}", e),
        }
    }
}

impl Error for ApiErr {}

/// Result type for API
pub type ApiResult<T> = Result<T, ApiErr>;
