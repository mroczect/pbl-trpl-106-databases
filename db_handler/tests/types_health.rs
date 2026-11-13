#![allow(unused_crate_dependencies, clippy::unwrap_used, clippy::expect_used)]

use db_handler::json::{from_json, to_json};
use db_handler::types::core::{HealthDetails, HealthStatus};
use serde_json::json;

// ===== HealthStatus serialization =====

#[test]
fn health_status_serde_healthy() {
    assert_eq!(to_json(&HealthStatus::Healthy).unwrap(), json!("healthy"));
}

#[test]
fn health_status_serde_degraded() {
    assert_eq!(to_json(&HealthStatus::Degraded).unwrap(), json!("degraded"));
}

#[test]
fn health_status_serde_unhealthy() {
    assert_eq!(to_json(&HealthStatus::Unhealthy).unwrap(), json!("unhealthy"));
}


#[test]
fn health_status_deserialize_healthy() {
    let s: HealthStatus = from_json(&json!("healthy")).unwrap();
    assert_eq!(s, HealthStatus::Healthy);
}

#[test]
fn health_status_deserialize_degraded() {
    let s: HealthStatus = from_json(&json!("degraded")).unwrap();
    assert_eq!(s, HealthStatus::Degraded);
}

#[test]
fn health_status_deserialize_unhealthy() {
    let s: HealthStatus = from_json(&json!("unhealthy")).unwrap();
    assert_eq!(s, HealthStatus::Unhealthy);
}

#[test]
fn health_status_rejects_capitalized() {
    assert!(from_json::<HealthStatus>(&json!("Healthy")).is_err());
    assert!(from_json::<HealthStatus>(&json!("HEALTHY")).is_err());
    assert!(from_json::<HealthStatus>(&json!("Degraded")).is_err());
}

#[test]
fn health_status_rejects_unknown() {
    assert!(from_json::<HealthStatus>(&json!("unknown")).is_err());
    assert!(from_json::<HealthStatus>(&json!("ok")).is_err());
}

#[test]
fn health_status_rejects_non_string() {
    assert!(from_json::<HealthStatus>(&json!(1)).is_err());
    assert!(from_json::<HealthStatus>(&json!(null)).is_err());
    assert!(from_json::<HealthStatus>(&json!(true)).is_err());
    assert!(from_json::<HealthStatus>(&json!([])).is_err());
}


#[test]
fn health_status_eq() {
    assert_eq!(HealthStatus::Healthy, HealthStatus::Healthy);
    assert_ne!(HealthStatus::Healthy, HealthStatus::Degraded);
    assert_ne!(HealthStatus::Healthy, HealthStatus::Unhealthy);
    assert_ne!(HealthStatus::Degraded, HealthStatus::Unhealthy);
}

#[test]
fn health_status_copy() {
    let a = HealthStatus::Healthy;
    let b = a;
    assert_eq!(a, b);
}

#[test]
fn health_status_debug() {
    assert!(format!("{:?}", HealthStatus::Healthy).contains("Healthy"));
    assert!(format!("{:?}", HealthStatus::Degraded).contains("Degraded"));
    assert!(format!("{:?}", HealthStatus::Unhealthy).contains("Unhealthy"));
}


#[test]
fn health_details_serialize_with_message() {
    let d = HealthDetails {
        status: HealthStatus::Degraded,
        latency_ms: 250,
        server_version: "8.0.35".into(),
        message: Some("slow query".into()),
    };
    assert_eq!(
        to_json(&d).unwrap(),
        json!({
            "status": "degraded",
            "latency_ms": 250,
            "server_version": "8.0.35",
            "message": "slow query"
        })
    );
}

#[test]
fn health_details_serialize_omits_none_message() {
    let d = HealthDetails {
        status: HealthStatus::Healthy,
        latency_ms: 5,
        server_version: "8.0.35".into(),
        message: None,
    };
    let json = to_json(&d).unwrap();
    let obj = json.as_object().unwrap();
    assert!(!obj.contains_key("message"));
    assert_eq!(json["status"].as_str(), Some("healthy"));
    assert_eq!(json["latency_ms"].as_u64(), Some(5));
    assert_eq!(json["server_version"].as_str(), Some("8.0.35"));
}


#[test]
fn health_details_deserialize_with_message() {
    let d: HealthDetails = from_json(&json!({
        "status": "unhealthy",
        "latency_ms": 5000,
        "server_version": "5.7",
        "message": "connection lost"
    }))
    .unwrap();
    assert_eq!(d.status, HealthStatus::Unhealthy);
    assert_eq!(d.latency_ms, 5000);
    assert_eq!(d.server_version, "5.7");
    assert_eq!(d.message.as_deref(), Some("connection lost"));
}

#[test]
fn health_details_deserialize_without_message_field() {
    let d: HealthDetails = from_json(&json!({
        "status": "healthy",
        "latency_ms": 1,
        "server_version": "8.0"
    }))
    .unwrap();
    assert!(d.message.is_none());
}

#[test]
fn health_details_deserialize_null_message() {
    let d: HealthDetails = from_json(&json!({
        "status": "healthy",
        "latency_ms": 1,
        "server_version": "8.0",
        "message": null
    }))
    .unwrap();
    assert!(d.message.is_none());
}

#[test]
fn health_details_rejects_missing_status() {
    let r = from_json::<HealthDetails>(&json!({
        "latency_ms": 1,
        "server_version": "8.0"
    }));
    assert!(r.is_err());
}

#[test]
fn health_details_rejects_missing_latency() {
    let r = from_json::<HealthDetails>(&json!({
        "status": "healthy",
        "server_version": "8.0"
    }));
    assert!(r.is_err());
}

#[test]
fn health_details_rejects_missing_server_version() {
    let r = from_json::<HealthDetails>(&json!({
        "status": "healthy",
        "latency_ms": 1
    }));
    assert!(r.is_err());
}

#[test]
fn health_details_rejects_negative_latency() {
    let r = from_json::<HealthDetails>(&json!({
        "status": "healthy",
        "latency_ms": -1,
        "server_version": "8.0"
    }));
    assert!(r.is_err());
}

#[test]
fn health_details_rejects_float_latency() {
    let r = from_json::<HealthDetails>(&json!({
        "status": "healthy",
        "latency_ms": 1.5,
        "server_version": "8.0"
    }));
    assert!(r.is_err());
}

#[test]
fn health_details_rejects_invalid_status() {
    let r = from_json::<HealthDetails>(&json!({
        "status": "unknown",
        "latency_ms": 1,
        "server_version": "8.0"
    }));
    assert!(r.is_err());
}

#[test]
fn health_details_accepts_extra_fields() {
    let d: HealthDetails = from_json(&json!({
        "status": "healthy",
        "latency_ms": 1,
        "server_version": "8.0",
        "extra": "ignored",
        "another": 42
    }))
    .unwrap();
    assert_eq!(d.status, HealthStatus::Healthy);
}

#[test]
fn health_details_roundtrip() {
    let original = HealthDetails {
        status: HealthStatus::Degraded,
        latency_ms: 100,
        server_version: "8.0.35".into(),
        message: Some("minor lag".into()),
    };
    let json = to_json(&original).unwrap();
    let back: HealthDetails = from_json(&json).unwrap();
    assert_eq!(back.status, original.status);
    assert_eq!(back.latency_ms, original.latency_ms);
    assert_eq!(back.server_version, original.server_version);
    assert_eq!(back.message, original.message);
}

#[test]
fn health_details_clone() {
    let a = HealthDetails {
        status: HealthStatus::Healthy,
        latency_ms: 1,
        server_version: "8.0".into(),
        message: None,
    };
    let b = a.clone();
    assert_eq!(a.latency_ms, b.latency_ms);
}
