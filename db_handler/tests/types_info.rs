#![allow(unused_crate_dependencies, clippy::unwrap_used, clippy::expect_used)]

use db_handler::json::{from_json, to_json, Json};
use db_handler::types::core::DatabaseInfo;
use serde_json::json;

fn sample() -> DatabaseInfo {
    DatabaseInfo {
        database_name: "pbl_trpl_106".into(),
        server_version: "8.0.35".into(),
        pool_size: 10,
        pool_idle: 3,
        active_connections: 7,
    }
}

#[test]
fn database_info_serialize_exact_shape() {
    let j = to_json(&sample()).unwrap();
    let obj = j.as_object().unwrap();
    assert_eq!(obj.len(), 5);
    assert_eq!(
        obj.get("database_name").and_then(|v| v.as_str()),
        Some("pbl_trpl_106")
    );
    assert_eq!(
        obj.get("server_version").and_then(|v| v.as_str()),
        Some("8.0.35")
    );
    assert_eq!(obj.get("pool_size").and_then(Json::as_u64), Some(10));
    assert_eq!(obj.get("pool_idle").and_then(Json::as_u64), Some(3));
    assert_eq!(
        obj.get("active_connections").and_then(Json::as_u64),
        Some(7)
    );
}

#[test]
fn database_info_deserialize_exact() {
    let info: DatabaseInfo = from_json(&json!({
        "database_name": "pbl_trpl_106",
        "server_version": "8.0.35",
        "pool_size": 10,
        "pool_idle": 3,
        "active_connections": 7
    }))
    .unwrap();
    assert_eq!(info.database_name, "pbl_trpl_106");
    assert_eq!(info.server_version, "8.0.35");
    assert_eq!(info.pool_size, 10);
    assert_eq!(info.pool_idle, 3);
    assert_eq!(info.active_connections, 7);
}

#[test]
fn database_info_rejects_missing_database_name() {
    let r = from_json::<DatabaseInfo>(&json!({
        "server_version": "8.0",
        "pool_size": 10,
        "pool_idle": 3,
        "active_connections": 7
    }));
    assert!(r.is_err());
}

#[test]
fn database_info_rejects_missing_server_version() {
    let r = from_json::<DatabaseInfo>(&json!({
        "database_name": "db",
        "pool_size": 10,
        "pool_idle": 3,
        "active_connections": 7
    }));
    assert!(r.is_err());
}

#[test]
fn database_info_rejects_missing_pool_size() {
    let r = from_json::<DatabaseInfo>(&json!({
        "database_name": "db",
        "server_version": "8.0",
        "pool_idle": 3,
        "active_connections": 7
    }));
    assert!(r.is_err());
}

#[test]
fn database_info_rejects_missing_pool_idle() {
    let r = from_json::<DatabaseInfo>(&json!({
        "database_name": "db",
        "server_version": "8.0",
        "pool_size": 10,
        "active_connections": 7
    }));
    assert!(r.is_err());
}

#[test]
fn database_info_rejects_missing_active_connections() {
    let r = from_json::<DatabaseInfo>(&json!({
        "database_name": "db",
        "server_version": "8.0",
        "pool_size": 10,
        "pool_idle": 3
    }));
    assert!(r.is_err());
}

#[test]
fn database_info_rejects_negative_pool_size() {
    let r = from_json::<DatabaseInfo>(&json!({
        "database_name": "db",
        "server_version": "8.0",
        "pool_size": -1,
        "pool_idle": 0,
        "active_connections": 0
    }));
    assert!(r.is_err());
}

#[test]
fn database_info_rejects_negative_pool_idle() {
    let r = from_json::<DatabaseInfo>(&json!({
        "database_name": "db",
        "server_version": "8.0",
        "pool_size": 10,
        "pool_idle": -1,
        "active_connections": 0
    }));
    assert!(r.is_err());
}

#[test]
fn database_info_rejects_negative_active_connections() {
    let r = from_json::<DatabaseInfo>(&json!({
        "database_name": "db",
        "server_version": "8.0",
        "pool_size": 10,
        "pool_idle": 0,
        "active_connections": -1
    }));
    assert!(r.is_err());
}

#[test]
fn database_info_rejects_pool_size_overflow() {
    let r = from_json::<DatabaseInfo>(&json!({
        "database_name": "db",
        "server_version": "8.0",
        "pool_size": 4_294_967_296_u64,
        "pool_idle": 0,
        "active_connections": 0
    }));
    assert!(r.is_err());
}

#[test]
fn database_info_rejects_pool_idle_overflow() {
    let r = from_json::<DatabaseInfo>(&json!({
        "database_name": "db",
        "server_version": "8.0",
        "pool_size": 0,
        "pool_idle": 4_294_967_296_u64,
        "active_connections": 0
    }));
    assert!(r.is_err());
}

#[test]
fn database_info_rejects_active_connections_overflow() {
    let r = from_json::<DatabaseInfo>(&json!({
        "database_name": "db",
        "server_version": "8.0",
        "pool_size": 0,
        "pool_idle": 0,
        "active_connections": 4_294_967_296_u64
    }));
    assert!(r.is_err());
}

#[test]
fn database_info_rejects_string_pool_size() {
    let r = from_json::<DatabaseInfo>(&json!({
        "database_name": "db",
        "server_version": "8.0",
        "pool_size": "10",
        "pool_idle": 0,
        "active_connections": 0
    }));
    assert!(r.is_err());
}

#[test]
fn database_info_rejects_float_pool_size() {
    let r = from_json::<DatabaseInfo>(&json!({
        "database_name": "db",
        "server_version": "8.0",
        "pool_size": 10.5,
        "pool_idle": 0,
        "active_connections": 0
    }));
    assert!(r.is_err());
}

#[test]
fn database_info_accepts_extra_fields() {
    let info: DatabaseInfo = from_json(&json!({
        "database_name": "db",
        "server_version": "8.0",
        "pool_size": 10,
        "pool_idle": 3,
        "active_connections": 7,
        "extra": true,
        "nested": {"a": 1}
    }))
    .unwrap();
    assert_eq!(info.pool_size, 10);
}

#[test]
fn database_info_accepts_all_zeros() {
    let info: DatabaseInfo = from_json(&json!({
        "database_name": "",
        "server_version": "",
        "pool_size": 0,
        "pool_idle": 0,
        "active_connections": 0
    }))
    .unwrap();
    assert_eq!(info.database_name, "");
    assert_eq!(info.pool_size, 0);
}

#[test]
fn database_info_accepts_u32_max() {
    let info: DatabaseInfo = from_json(&json!({
        "database_name": "db",
        "server_version": "8.0",
        "pool_size": u32::MAX,
        "pool_idle": u32::MAX,
        "active_connections": u32::MAX
    }))
    .unwrap();
    assert_eq!(info.pool_size, u32::MAX);
    assert_eq!(info.pool_idle, u32::MAX);
    assert_eq!(info.active_connections, u32::MAX);
}

#[test]
fn database_info_accepts_unicode_strings() {
    let info: DatabaseInfo = from_json(&json!({
        "database_name": "データベース",
        "server_version": "8.0-β",
        "pool_size": 1,
        "pool_idle": 1,
        "active_connections": 0
    }))
    .unwrap();
    assert_eq!(info.database_name, "データベース");
    assert_eq!(info.server_version, "8.0-β");
}

#[test]
fn database_info_rejects_null_root() {
    let r = from_json::<DatabaseInfo>(&json!(null));
    assert!(r.is_err());
}

#[test]
fn database_info_rejects_array_root() {
    let r = from_json::<DatabaseInfo>(&json!([1, 2, 3, 4, 5]));
    assert!(r.is_err());
}

#[test]
fn database_info_roundtrip() {
    let original = sample();
    let back: DatabaseInfo = from_json(&to_json(&original).unwrap()).unwrap();
    assert_eq!(back.database_name, original.database_name);
    assert_eq!(back.server_version, original.server_version);
    assert_eq!(back.pool_size, original.pool_size);
    assert_eq!(back.pool_idle, original.pool_idle);
    assert_eq!(back.active_connections, original.active_connections);
}

#[test]
fn database_info_clone_preserves_all_fields() {
    let a = sample();
    let b = a.clone();
    assert_eq!(a.database_name, b.database_name);
    assert_eq!(a.server_version, b.server_version);
    assert_eq!(a.pool_size, b.pool_size);
    assert_eq!(a.pool_idle, b.pool_idle);
    assert_eq!(a.active_connections, b.active_connections);
}

#[test]
fn database_info_debug_contains_type_name() {
    assert!(format!("{:?}", sample()).contains("DatabaseInfo"));
}
