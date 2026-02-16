use parro::command::Command;
use parro::error::ClientError;

#[test]
fn test_parse_get_success() {
    let result = Command::from_str("GET key").unwrap();
    assert_eq!(result, Command::Get("key"));
}

#[test]
fn test_parse_get_no_key() {
    let result = Command::from_str("GET");
    assert_eq!(result, Err(ClientError::WrongNumberOfArguments));
}

#[test]
fn test_parse_get_too_many_args() {
    let result = Command::from_str("GET key value");
    assert_eq!(result, Err(ClientError::WrongNumberOfArguments));
}

#[test]
fn test_parse_set_success() {
    let result = Command::from_str("SET key value").unwrap();
    assert_eq!(result, Command::Set("key", "value"));
}

#[test]
fn test_parse_set_no_args() {
    let result = Command::from_str("SET");
    assert_eq!(result, Err(ClientError::WrongNumberOfArguments));
}

#[test]
fn test_parse_set_one_arg() {
    let result = Command::from_str("SET key");
    assert_eq!(result, Err(ClientError::WrongNumberOfArguments));
}

#[test]
fn test_parse_set_too_many_args() {
    let result = Command::from_str("SET key value extra");
    assert_eq!(result, Err(ClientError::WrongNumberOfArguments));
}

#[test]
fn test_parse_unknown_command() {
    let result = Command::from_str("UNKNOWN");
    assert_eq!(result, Err(ClientError::UnknownCommand));
}

#[test]
fn test_parse_empty_string() {
    let result = Command::from_str("");
    assert_eq!(result, Err(ClientError::UnknownCommand));
}

#[test]
fn test_parse_set_with_multiple_spaces() {
    let result = Command::from_str("SET   key   value").unwrap();
    assert_eq!(result, Command::Set("key", "value"));
}

#[test]
fn test_parse_get_with_multiple_spaces() {
    let result = Command::from_str("GET   key").unwrap();
    assert_eq!(result, Command::Get("key"));
}

#[test]
fn test_case_sensitive_command() {
    let result = Command::from_str("get key");
    assert_eq!(result, Err(ClientError::UnknownCommand));
}
