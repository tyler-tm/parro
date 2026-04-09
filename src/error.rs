use std::error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageError {
    LimitExceeded,
    NotFound,
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StorageError::LimitExceeded => write!(f, "storage limit exceeded"),
            StorageError::NotFound => write!(f, "key not found"),
        }
    }
}

impl error::Error for StorageError {}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;
