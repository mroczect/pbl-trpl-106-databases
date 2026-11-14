#![allow(unused_crate_dependencies, clippy::unwrap_used, clippy::expect_used)]

use db_handler::validation::validate_identifier;
use db_handler::DbError;

#[test]
fn accepts_simple_name() {
    assert!(validate_identifier("users").is_ok());
    assert!(validate_identifier("user_table").is_ok());
    assert!(validate_identifier("_private").is_ok());
    assert!(validate_identifier("a1_b2").is_ok());
}

#[test]
fn rejects_empty() {
    let err = validate_identifier("").unwrap_err();
    assert!(matches!(err, DbError::Validation { .. }));
}

#[test]
fn rejects_starting_with_digit() {
    assert!(validate_identifier("1users").is_err());
}

#[test]
fn rejects_special_chars() {
    assert!(validate_identifier("user-name").is_err());
    assert!(validate_identifier("user name").is_err());
    assert!(validate_identifier("user;drop").is_err());
    assert!(validate_identifier("user'").is_err());
}

#[test]
fn rejects_too_long() {
    let name = "a".repeat(65);
    assert!(validate_identifier(&name).is_err());
}

#[test]
fn accepts_max_length() {
    let name = "a".repeat(64);
    assert!(validate_identifier(&name).is_ok());
}
