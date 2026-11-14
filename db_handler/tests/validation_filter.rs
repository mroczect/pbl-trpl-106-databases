#![allow(unused_crate_dependencies, clippy::unwrap_used, clippy::expect_used)]

use db_handler::types::core::{Condition, Filter};
use db_handler::validation::validate_filter;
use db_handler::DbError;

#[test]
fn accepts_empty_filter() {
    assert!(validate_filter(&Filter::new()).is_ok());
}

#[test]
fn accepts_valid_filter() {
    let f = Filter::new().add(Condition::eq("id", 1i64));
    assert!(validate_filter(&f).is_ok());
}

#[test]
fn rejects_invalid_column() {
    let f = Filter::new().add(Condition::eq("1bad", 1i64));
    assert!(validate_filter(&f).is_err());
}

#[test]
fn rejects_nested_empty_and() {
    let f = Filter::new().add(Condition::And(Vec::new()));
    assert!(validate_filter(&f).is_err());
}

#[test]
fn rejects_nested_empty_or() {
    let f = Filter::new().add(Condition::Or(Vec::new()));
    assert!(validate_filter(&f).is_err());
}

#[test]
fn accepts_nested_valid() {
    let f = Filter::new().add(Condition::And(vec![
        Condition::eq("a", 1i64),
        Condition::eq("b", 2i64),
    ]));
    assert!(validate_filter(&f).is_ok());
}

#[test]
fn rejects_nested_invalid() {
    let f = Filter::new().add(Condition::Or(vec![
        Condition::eq("good", 1i64),
        Condition::eq("1bad", 2i64),
    ]));
    let err = validate_filter(&f).unwrap_err();
    assert!(matches!(err, DbError::Validation { .. }));
}
