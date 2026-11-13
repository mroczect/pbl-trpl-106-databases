#![allow(unused_crate_dependencies, clippy::unwrap_used, clippy::expect_used)]

use db_handler::json::{from_json, to_json};
use db_handler::types::core::SortDirection;
use serde_json::json;

#[test]
fn sort_direction_default_is_asc() {
    assert_eq!(SortDirection::default(), SortDirection::Asc);
}

#[test]
fn sort_direction_serde_asc() {
    assert_eq!(to_json(&SortDirection::Asc).unwrap(), json!("asc"));
}

#[test]
fn sort_direction_serde_desc() {
    assert_eq!(to_json(&SortDirection::Desc).unwrap(), json!("desc"));
}

#[test]
fn sort_direction_deserialize_asc() {
    assert_eq!(
        from_json::<SortDirection>(&json!("asc")).unwrap(),
        SortDirection::Asc
    );
}

#[test]
fn sort_direction_deserialize_desc() {
    assert_eq!(
        from_json::<SortDirection>(&json!("desc")).unwrap(),
        SortDirection::Desc
    );
}

#[test]
fn sort_direction_rejects_capitalized() {
    assert!(from_json::<SortDirection>(&json!("Asc")).is_err());
    assert!(from_json::<SortDirection>(&json!("Desc")).is_err());
    assert!(from_json::<SortDirection>(&json!("ASC")).is_err());
}

#[test]
fn sort_direction_rejects_longer_forms() {
    assert!(from_json::<SortDirection>(&json!("ascending")).is_err());
    assert!(from_json::<SortDirection>(&json!("descending")).is_err());
}

#[test]
fn sort_direction_rejects_non_string() {
    assert!(from_json::<SortDirection>(&json!(1)).is_err());
    assert!(from_json::<SortDirection>(&json!(null)).is_err());
    assert!(from_json::<SortDirection>(&json!(true)).is_err());
}

#[test]
fn sort_direction_eq() {
    assert_eq!(SortDirection::Asc, SortDirection::Asc);
    assert_eq!(SortDirection::Desc, SortDirection::Desc);
    assert_ne!(SortDirection::Asc, SortDirection::Desc);
}

#[test]
fn sort_direction_copy() {
    let a = SortDirection::Asc;
    let b = a;
    assert_eq!(a, b);
}

#[test]
fn sort_direction_debug() {
    assert!(format!("{:?}", SortDirection::Asc).contains("Asc"));
    assert!(format!("{:?}", SortDirection::Desc).contains("Desc"));
}

#[test]
fn sort_direction_roundtrip() {
    for original in [SortDirection::Asc, SortDirection::Desc] {
        let back: SortDirection = from_json(&to_json(&original).unwrap()).unwrap();
        assert_eq!(back, original);
    }
}
