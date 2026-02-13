use crate::error::ClientError;

#[derive(Debug)]
pub enum Command<'a> {
    Get(&'a str),
    Set(&'a str, &'a str),
}

impl<'a> Command<'a> {
    pub fn from_str(s: &'a str) -> Result<Command<'a>, ClientError> {
        let mut parts = s.trim().split_whitespace();
        match parts.next() {
            Some("GET") => {
                if let Some(key) = parts.next() {
                    if parts.next().is_none() {
                        Ok(Command::Get(key))
                    } else {
                        Err(ClientError::WrongNumberOfArguments)
                    }
                } else {
                    Err(ClientError::WrongNumberOfArguments)
                }
            }
            Some("SET") => {
                if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                    if parts.next().is_none() {
                        Ok(Command::Set(key, value))
                    } else {
                        Err(ClientError::WrongNumberOfArguments)
                    }
                } else {
                    Err(ClientError::WrongNumberOfArguments)
                }
            }
            _ => Err(ClientError::UnknownCommand),
        }
    }
}
