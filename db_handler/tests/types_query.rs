#![allow(unused_crate_dependencies, clippy::unwrap_used, clippy::expect_used)]

use db_handler::types::core::{ColumnValue, Query, Value};

#[test]
fn query_new_has_no_params() {
    let q = Query::new("SELECT 1");
    assert_eq!(q.sql, "SELECT 1");
    assert_eq!(q.param_count(), 0);
    assert!(!q.is_empty());
}

#[test]
fn query_bind_adds_params_in_order() {
    let q = Query::new("INSERT INTO t VALUES (?, ?, ?)")
        .bind("a")
        .bind(42i64)
        .bind(true);
    assert_eq!(q.params.len(), 3);
    assert_eq!(q.params.first(), Some(&Value::String("a".into())));
    assert_eq!(q.params.get(1), Some(&Value::Int(42)));
    assert_eq!(q.params.get(2), Some(&Value::Bool(true)));
}

#[test]
fn query_empty_sql_is_empty() {
    assert!(Query::new("").is_empty());
}

#[test]
fn query_bind_accepts_option() {
    let q = Query::new("x").bind(Some(1i64)).bind(Option::<i64>::None);
    assert_eq!(q.params.first(), Some(&Value::Int(1)));
    assert_eq!(q.params.get(1), Some(&Value::Null));
}

#[test]
fn column_value_new() {
    let cv = ColumnValue::new("name", "Alice");
    assert_eq!(cv.column, "name");
    assert_eq!(cv.value, Value::String("Alice".into()));
}

#[test]
fn query_clone_and_eq() {
    let a = Query::new("x").bind(1i64);
    let b = a.clone();
    assert_eq!(a, b);
}
