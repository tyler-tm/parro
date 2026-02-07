use crate::error::ClientError;

#[derive(Debug)]
pub enum Command {
    Get(String),
    Set(String, String),
}

impl Command {
    pub fn from_str(s: &str) -> Result<Command, ClientError> {
        let parts: Vec<&str> = s.trim().split(' ').collect();
        match parts.get(0).map(|s| *s) {
            Some("GET") => {
                if parts.len() == 2 {
                    Ok(Command::Get(parts[1].to_string()))
                } else {
                    Err(ClientError::WrongNumberOfArguments)
                }
            }
            Some("SET") => {
                if parts.len() == 3 {
                    Ok(Command::Set(parts[1].to_string(), parts[2].to_string()))
                } else {
                    Err(ClientError::WrongNumberOfArguments)
                }
            }
            _ => Err(ClientError::UnknownCommand),
        }
    }
}
