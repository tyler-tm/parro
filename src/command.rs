use crate::error::ClientError;

#[derive(Debug, PartialEq)]
pub enum Command<'a> {
    Get(&'a str),
    Set(&'a str, &'a str),
    Delete(&'a str),
}

impl<'a> Command<'a> {
    pub fn parse(s: &'a str) -> Result<Command<'a>, ClientError> {
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
            Some("DELETE") => {
                if let Some(key) = parts.next() {
                    if parts.next().is_none() {
                        Ok(Command::Delete(key))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_get_success() {
        let result = Command::parse("GET key").unwrap();
        assert_eq!(result, Command::Get("key"));
    }

    #[test]
    fn parse_get_no_key() {
        let result = Command::parse("GET");
        assert_eq!(result, Err(ClientError::WrongNumberOfArguments));
    }

    #[test]
    fn parse_get_too_many_args() {
        let result = Command::parse("GET key value");
        assert_eq!(result, Err(ClientError::WrongNumberOfArguments));
    }

    #[test]
    fn parse_set_success() {
        let result = Command::parse("SET key value").unwrap();
        assert_eq!(result, Command::Set("key", "value"));
    }

    #[test]
    fn parse_set_no_args() {
        let result = Command::parse("SET");
        assert_eq!(result, Err(ClientError::WrongNumberOfArguments));
    }

    #[test]
    fn parse_set_one_arg() {
        let result = Command::parse("SET key");
        assert_eq!(result, Err(ClientError::WrongNumberOfArguments));
    }

    #[test]
    fn parse_set_too_many_args() {
        let result = Command::parse("SET key value extra");
        assert_eq!(result, Err(ClientError::WrongNumberOfArguments));
    }

    #[test]
    fn parse_unknown_command() {
        let result = Command::parse("UNKNOWN");
        assert_eq!(result, Err(ClientError::UnknownCommand));
    }

    #[test]
    fn parse_empty_string() {
        let result = Command::parse("");
        assert_eq!(result, Err(ClientError::UnknownCommand));
    }

    #[test]
    fn parse_set_with_multiple_spaces() {
        let result = Command::parse("SET   key   value").unwrap();
        assert_eq!(result, Command::Set("key", "value"));
    }

    #[test]
    fn parse_get_with_multiple_spaces() {
        let result = Command::parse("GET   key").unwrap();
        assert_eq!(result, Command::Get("key"));
    }

    #[test]
    fn case_sensitive_command() {
        let result = Command::parse("get key");
        assert_eq!(result, Err(ClientError::UnknownCommand));
    }

    #[test]
    fn parse_delete_success() {
        let result = Command::parse("DELETE key").unwrap();
        assert_eq!(result, Command::Delete("key"));
    }

    #[test]
    fn parse_delete_no_key() {
        let result = Command::parse("DELETE");
        assert_eq!(result, Err(ClientError::WrongNumberOfArguments));
    }

    #[test]
    fn parse_delete_too_many_args() {
        let result = Command::parse("DELETE key value");
        assert_eq!(result, Err(ClientError::WrongNumberOfArguments));
    }

    #[test]
    fn parse_delete_with_multiple_spaces() {
        let result = Command::parse("DELETE   key").unwrap();
        assert_eq!(result, Command::Delete("key"));
    }
}
