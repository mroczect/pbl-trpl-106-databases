#![allow(unused_crate_dependencies, clippy::unwrap_used, clippy::expect_used)]

use db_handler::PoolConfig;

#[test]
fn default_is_valid() {
    assert!(PoolConfig::default().validate().is_ok());
}

#[test]
fn default_values_are_stable() {
    let c = PoolConfig::default();
    assert_eq!(c.max_connections, 10);
    assert_eq!(c.min_connections, 0);
    assert_eq!(c.acquire_timeout_ms, 30_000);
    assert_eq!(c.connect_timeout_ms, 10_000);
    assert_eq!(c.max_lifetime_secs, 1_800);
    assert_eq!(c.idle_timeout_secs, 600);
}

#[test]
fn rejects_zero_max_connections() {
    let c = PoolConfig {
        max_connections: 0,
        ..Default::default()
    };
    assert!(c.validate().is_err());
}

#[test]
fn rejects_min_greater_than_max() {
    let c = PoolConfig {
        max_connections: 5,
        min_connections: 6,
        ..Default::default()
    };
    assert!(c.validate().is_err());
}

#[test]
fn accepts_min_equal_to_max() {
    let c = PoolConfig {
        max_connections: 5,
        min_connections: 5,
        ..Default::default()
    };
    assert!(c.validate().is_ok());
}

#[test]
fn rejects_zero_acquire_timeout() {
    let c = PoolConfig {
        acquire_timeout_ms: 0,
        ..Default::default()
    };
    assert!(c.validate().is_err());
}

#[test]
fn rejects_zero_connect_timeout() {
    let c = PoolConfig {
        connect_timeout_ms: 0,
        ..Default::default()
    };
    assert!(c.validate().is_err());
}

#[test]
fn zero_lifetimes_are_allowed() {
    let c = PoolConfig {
        max_lifetime_secs: 0,
        idle_timeout_secs: 0,
        ..Default::default()
    };
    assert!(c.validate().is_ok());
}

#[test]
fn copy_semantics() {
    let a = PoolConfig::default();
    let b = a;
    assert_eq!(a, b);
}

#[test]
fn serializes_and_roundtrips() {
    let c = PoolConfig {
        max_connections: 25,
        min_connections: 5,
        acquire_timeout_ms: 15_000,
        connect_timeout_ms: 5_000,
        max_lifetime_secs: 3_600,
        idle_timeout_secs: 300,
    };
    let json = serde_json::to_value(c).unwrap();
    let max_conn = json
        .get("max_connections")
        .and_then(serde_json::Value::as_u64);
    assert_eq!(max_conn, Some(25));
    let back: PoolConfig = serde_json::from_value(json).unwrap();
    assert_eq!(back, c);
}
