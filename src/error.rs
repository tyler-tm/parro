use std::error;
use std::fmt;

#[derive(Debug, Clone)]
pub enum ClientError {
    UnknownCommand,
    WrongNumberOfArguments,
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ClientError::UnknownCommand => write!(f, "Error: unknown command"),
            ClientError::WrongNumberOfArguments => {
                write!(f, "Error: wrong number of arguments for command")
            }
        }
    }
}

impl error::Error for ClientError {}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;
