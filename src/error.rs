use std::error;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum ClientError {
    UnknownCommand,
    WrongNumberOfArguments,
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ClientError::UnknownCommand => write!(f, "unknown command"),
            ClientError::WrongNumberOfArguments => {
                write!(f, "wrong number of arguments for command")
            }
        }
    }
}

impl error::Error for ClientError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageError {
    LimitExceeded,
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StorageError::LimitExceeded => write!(f, "storage limit exceeded"),
        }
    }
}

impl error::Error for StorageError {}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;
