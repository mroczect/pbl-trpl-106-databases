#![allow(unused_crate_dependencies, clippy::unwrap_used, clippy::expect_used)]

use db_handler::types::core::IsolationLevel;

#[test]
fn isolation_level_variants_distinct() {
    assert_ne!(IsolationLevel::ReadCommitted, IsolationLevel::Serializable);
}

#[test]
fn isolation_level_copy() {
    let a = IsolationLevel::RepeatableRead;
    let b = a;
    assert_eq!(a, b);
}
