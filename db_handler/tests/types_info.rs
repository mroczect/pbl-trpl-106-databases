#![allow(unused_crate_dependencies, clippy::unwrap_used, clippy::expect_used)]

use db_handler::json::{from_json, to_json};
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
fn database_info_serialize_all_fields() {
    assert_eq!(
        to_json(&sample()).unwrap(),
        json!({
            "database_name": "pbl_trpl_106",
            "server_version": "8.0.35",
            "pool_size": 10,
            "pool_idle": 3,
            "active_connections": 7
        })
    );
}

#[test]
fn database_info_roundtrip() {
    let original = sample();
    let json = to_json(&original).unwrap();
    let back: DatabaseInfo = from_json(&json).unwrap();
    assert_eq!(back.database_name, original.database_name);
    assert_eq!(back.server_version, original.server_version);
    assert_eq!(back.pool_size, original.pool_size);
    assert_eq!(back.pool_idle, original.pool_idle);
    assert_eq!(back.active_connections, original.active_connections);
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
fn database_info_rejects_overflow_pool_size() {
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
fn database_info_accepts_extra_fields() {
    let info: DatabaseInfo = from_json(&json!({
        "database_name": "db",
        "server_version": "8.0",
        "pool_size": 10,
        "pool_idle": 3,
        "active_connections": 7,
        "extra": true
    }))
    .unwrap();
    assert_eq!(info.pool_size, 10);
}

#[test]
fn database_info_clone_and_debug() {
    let a = sample();
    let b = a.clone();
    assert_eq!(a.pool_size, b.pool_size);
    assert!(format!("{:?}", a).contains("DatabaseInfo"));
}
