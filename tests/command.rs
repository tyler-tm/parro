use parro::command::Command;
use parro::error::ClientError;

#[test]
fn parse_get_success() {
    let result = Command::from_str("GET key").unwrap();
    assert_eq!(result, Command::Get("key"));
}

#[test]
fn parse_get_no_key() {
    let result = Command::from_str("GET");
    assert_eq!(result, Err(ClientError::WrongNumberOfArguments));
}

#[test]
fn parse_get_too_many_args() {
    let result = Command::from_str("GET key value");
    assert_eq!(result, Err(ClientError::WrongNumberOfArguments));
}

#[test]
fn parse_set_success() {
    let result = Command::from_str("SET key value").unwrap();
    assert_eq!(result, Command::Set("key", "value"));
}

#[test]
fn parse_set_no_args() {
    let result = Command::from_str("SET");
    assert_eq!(result, Err(ClientError::WrongNumberOfArguments));
}

#[test]
fn parse_set_one_arg() {
    let result = Command::from_str("SET key");
    assert_eq!(result, Err(ClientError::WrongNumberOfArguments));
}

#[test]
fn parse_set_too_many_args() {
    let result = Command::from_str("SET key value extra");
    assert_eq!(result, Err(ClientError::WrongNumberOfArguments));
}

#[test]
fn parse_unknown_command() {
    let result = Command::from_str("UNKNOWN");
    assert_eq!(result, Err(ClientError::UnknownCommand));
}

#[test]
fn parse_empty_string() {
    let result = Command::from_str("");
    assert_eq!(result, Err(ClientError::UnknownCommand));
}

#[test]
fn parse_set_with_multiple_spaces() {
    let result = Command::from_str("SET   key   value").unwrap();
    assert_eq!(result, Command::Set("key", "value"));
}

#[test]
fn parse_get_with_multiple_spaces() {
    let result = Command::from_str("GET   key").unwrap();
    assert_eq!(result, Command::Get("key"));
}

#[test]
fn case_sensitive_command() {
    let result = Command::from_str("get key");
    assert_eq!(result, Err(ClientError::UnknownCommand));
}

#[test]
fn parse_delete_success() {
    let result = Command::from_str("DELETE key").unwrap();
    assert_eq!(result, Command::Delete("key"));
}

#[test]
fn parse_delete_no_key() {
    let result = Command::from_str("DELETE");
    assert_eq!(result, Err(ClientError::WrongNumberOfArguments));
}

#[test]
fn parse_delete_too_many_args() {
    let result = Command::from_str("DELETE key value");
    assert_eq!(result, Err(ClientError::WrongNumberOfArguments));
}

#[test]
fn parse_delete_with_multiple_spaces() {
    let result = Command::from_str("DELETE   key").unwrap();
    assert_eq!(result, Command::Delete("key"));
}
