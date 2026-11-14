#![allow(unused_crate_dependencies, clippy::unwrap_used, clippy::expect_used)]

use db_handler::types::core::{Condition, DeleteInput, Filter, InsertInput, UpdateInput, Value};

#[test]
fn filter_default_is_empty() {
    assert!(Filter::new().is_empty());
}

#[test]
fn filter_add_condition() {
    let f = Filter::new().add(Condition::eq("id", 1i64));
    assert_eq!(f.conditions.len(), 1);
}

#[test]
fn filter_add_multiple() {
    let f = Filter::new()
        .add(Condition::eq("a", 1i64))
        .add(Condition::eq("b", 2i64));
    assert_eq!(f.conditions.len(), 2);
}

#[test]
fn condition_eq_builds_correctly() {
    let c = Condition::eq("id", 42i64);
    assert_eq!(
        c,
        Condition::Eq {
            column: "id".into(),
            value: Value::Int(42),
        }
    );
}

#[test]
fn condition_is_null() {
    let c = Condition::is_null("deleted_at");
    assert_eq!(
        c,
        Condition::IsNull {
            column: "deleted_at".into(),
        }
    );
}

#[test]
fn condition_and_or_nested() {
    let inner = Condition::And(vec![Condition::eq("a", 1i64), Condition::eq("b", 2i64)]);
    let outer = Condition::Or(vec![inner, Condition::eq("c", 3i64)]);
    assert!(matches!(outer, Condition::Or(_)));
}

#[test]
fn insert_input_new_and_set() {
    let i = InsertInput::new("users")
        .set("name", "Alice")
        .set("age", 30i64);
    assert_eq!(i.table, "users");
    assert_eq!(i.values.len(), 2);
    assert_eq!(i.values.first().map(|v| v.column.as_str()), Some("name"));
    assert_eq!(i.values.get(1).map(|v| v.column.as_str()), Some("age"));
}

#[test]
fn update_input_new_set_where() {
    let f = Filter::new().add(Condition::eq("id", 1i64));
    let u = UpdateInput::new("users").set("name", "Bob").where_(f);
    assert_eq!(u.table, "users");
    assert_eq!(u.set.len(), 1);
    assert_eq!(u.filter.conditions.len(), 1);
}

#[test]
fn delete_input_new_where() {
    let f = Filter::new().add(Condition::eq("id", 1i64));
    let d = DeleteInput::new("users").where_(f);
    assert_eq!(d.table, "users");
    assert_eq!(d.filter.conditions.len(), 1);
}
