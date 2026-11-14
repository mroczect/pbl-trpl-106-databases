#![allow(unused_crate_dependencies, clippy::unwrap_used, clippy::expect_used)]

use db_handler::types::core::{Column, Value};

#[test]
fn value_null_is_null() {
    assert!(Value::Null.is_null());
    assert!(!Value::Int(1).is_null());
}

#[test]
fn value_as_str_returns_string_and_timestamp() {
    assert_eq!(Value::String("a".into()).as_str(), Some("a"));
    assert_eq!(Value::Timestamp("2026".into()).as_str(), Some("2026"));
    assert!(Value::Int(1).as_str().is_none());
}

#[test]
fn value_as_i64_only_for_int() {
    assert_eq!(Value::Int(42).as_i64(), Some(42));
    assert!(Value::Uint(42).as_i64().is_none());
}

#[test]
fn value_as_u64_only_for_uint() {
    assert_eq!(Value::Uint(42).as_u64(), Some(42));
    assert!(Value::Int(42).as_u64().is_none());
}

#[test]
fn value_as_bool_only_for_bool() {
    assert_eq!(Value::Bool(true).as_bool(), Some(true));
    assert!(Value::Int(1).as_bool().is_none());
}

#[test]
fn value_as_f64_only_for_float() {
    assert_eq!(Value::Float(1.5).as_f64(), Some(1.5));
    assert!(Value::Int(1).as_f64().is_none());
}

#[test]
fn value_as_bytes_only_for_bytes() {
    assert_eq!(Value::Bytes(vec![1, 2]).as_bytes(), Some(&[1u8, 2u8][..]));
    assert!(Value::String("x".into()).as_bytes().is_none());
}

#[test]
fn value_from_conversions() {
    assert_eq!(Value::from(true), Value::Bool(true));
    assert_eq!(Value::from(42i8), Value::Int(42));
    assert_eq!(Value::from(42i32), Value::Int(42));
    assert_eq!(Value::from(42i64), Value::Int(42));
    assert_eq!(Value::from(42u8), Value::Uint(42));
    assert_eq!(Value::from(42u64), Value::Uint(42));
    assert_eq!(Value::from(1.5f32), Value::Float(1.5));
    assert_eq!(Value::from(1.5f64), Value::Float(1.5));
    assert_eq!(Value::from("hi"), Value::String("hi".into()));
    assert_eq!(Value::from(String::from("hi")), Value::String("hi".into()));
    assert_eq!(Value::from(vec![1u8, 2u8]), Value::Bytes(vec![1, 2]));
}

#[test]
fn value_from_option() {
    assert_eq!(Value::from(Some(42i64)), Value::Int(42));
    assert_eq!(Value::from(Option::<i64>::None), Value::Null);
}

#[test]
fn value_serializes_to_json() {
    let json = serde_json::to_value(Value::Int(42)).unwrap();
    assert_eq!(json, serde_json::json!(42));

    let json = serde_json::to_value(Value::Null).unwrap();
    assert!(json.is_null());

    let json = serde_json::to_value(Value::String("a".into())).unwrap();
    assert_eq!(json, serde_json::json!("a"));
}

#[test]
fn column_new() {
    let c = Column::new("id", "BIGINT", false);
    assert_eq!(c.name, "id");
    assert_eq!(c.data_type, "BIGINT");
    assert!(!c.nullable);
}

#[test]
fn column_eq_and_clone() {
    let a = Column::new("a", "T", true);
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn value_partial_eq_handles_float() {
    assert_eq!(Value::Float(1.5), Value::Float(1.5));
    assert_ne!(Value::Float(1.5), Value::Float(2.5));
}

#[test]
fn row_trait_object_safe() {
    fn _takes(_r: &dyn db_handler::types::core::Row) {}
}
