#![allow(unused_crate_dependencies, clippy::unwrap_used, clippy::expect_used)]

use db_handler::types::core::{Column, ExecuteResult, QueryResult};

#[test]
fn execute_result_new() {
    let r = ExecuteResult::new(1, Some(7));
    assert_eq!(r.rows_affected, 1);
    assert_eq!(r.last_insert_id, Some(7));
}

#[test]
fn execute_result_default_is_zero() {
    let r = ExecuteResult::default();
    assert_eq!(r.rows_affected, 0);
    assert!(r.last_insert_id.is_none());
}

#[test]
fn query_result_new_empty() {
    let r = QueryResult::new(Vec::new(), Vec::new());
    assert_eq!(r.len(), 0);
    assert!(r.is_empty());
}

#[test]
fn query_result_new_with_columns() {
    let cols = vec![Column::new("id", "BIGINT", false)];
    let r = QueryResult::new(Vec::new(), cols);
    assert_eq!(r.columns.len(), 1);
    assert!(r.is_empty());
}
