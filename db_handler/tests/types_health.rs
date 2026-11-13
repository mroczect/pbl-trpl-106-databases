#![allow(unused_crate_dependencies, clippy::unwrap_used, clippy::expect_used)]

use db_handler::json::{from_json, to_json, Json};
use db_handler::types::core::{HealthDetails, HealthStatus};
use serde_json::json;

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
    assert_eq!(
        to_json(&HealthStatus::Unhealthy).unwrap(),
        json!("unhealthy")
    );
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
    assert!(from_json::<HealthStatus>(&json!("UNHEALTHY")).is_err());
}

#[test]
fn health_status_rejects_unknown() {
    assert!(from_json::<HealthStatus>(&json!("unknown")).is_err());
    assert!(from_json::<HealthStatus>(&json!("ok")).is_err());
    assert!(from_json::<HealthStatus>(&json!("")).is_err());
    assert!(from_json::<HealthStatus>(&json!("healthy ")).is_err());
}

#[test]
fn health_status_rejects_non_string() {
    assert!(from_json::<HealthStatus>(&json!(1)).is_err());
    assert!(from_json::<HealthStatus>(&json!(null)).is_err());
    assert!(from_json::<HealthStatus>(&json!(true)).is_err());
    assert!(from_json::<HealthStatus>(&json!([])).is_err());
    assert!(from_json::<HealthStatus>(&json!({})).is_err());
}

#[test]
fn health_status_eq_reflexive() {
    for variant in [
        HealthStatus::Healthy,
        HealthStatus::Degraded,
        HealthStatus::Unhealthy,
    ] {
        assert_eq!(variant, variant);
    }
}

#[test]
fn health_status_distinct_variants_not_equal() {
    assert_ne!(HealthStatus::Healthy, HealthStatus::Degraded);
    assert_ne!(HealthStatus::Healthy, HealthStatus::Unhealthy);
    assert_ne!(HealthStatus::Degraded, HealthStatus::Unhealthy);
}

#[test]
fn health_status_copy() {
    let a = HealthStatus::Degraded;
    let b = a;
    let c = a;
    assert_eq!(a, b);
    assert_eq!(a, c);
}

#[test]
fn health_status_debug_contains_variant_name() {
    assert!(format!("{:?}", HealthStatus::Healthy).contains("Healthy"));
    assert!(format!("{:?}", HealthStatus::Degraded).contains("Degraded"));
    assert!(format!("{:?}", HealthStatus::Unhealthy).contains("Unhealthy"));
}

#[test]
fn health_status_roundtrip_all() {
    for original in [
        HealthStatus::Healthy,
        HealthStatus::Degraded,
        HealthStatus::Unhealthy,
    ] {
        let back: HealthStatus = from_json(&to_json(&original).unwrap()).unwrap();
        assert_eq!(back, original);
    }
}

#[test]
fn health_details_serialize_with_message_exact() {
    let d = HealthDetails {
        status: HealthStatus::Degraded,
        latency_ms: 250,
        server_version: "8.0.35".into(),
        message: Some("slow query".into()),
    };
    let j = to_json(&d).unwrap();
    let obj = j.as_object().unwrap();
    assert_eq!(obj.len(), 4);
    assert_eq!(obj.get("status").and_then(|v| v.as_str()), Some("degraded"));
    assert_eq!(obj.get("latency_ms").and_then(Json::as_u64), Some(250));
    assert_eq!(
        obj.get("server_version").and_then(|v| v.as_str()),
        Some("8.0.35")
    );
    assert_eq!(
        obj.get("message").and_then(|v| v.as_str()),
        Some("slow query")
    );
}

#[test]
fn health_details_serialize_without_message_omits_key() {
    let d = HealthDetails {
        status: HealthStatus::Healthy,
        latency_ms: 5,
        server_version: "8.0.35".into(),
        message: None,
    };
    let j = to_json(&d).unwrap();
    let obj = j.as_object().unwrap();
    assert_eq!(obj.len(), 3);
    assert!(obj.get("message").is_none());
}

#[test]
fn health_details_serialize_empty_message_is_present() {
    let d = HealthDetails {
        status: HealthStatus::Healthy,
        latency_ms: 1,
        server_version: "8.0".into(),
        message: Some(String::new()),
    };
    let j = to_json(&d).unwrap();
    assert_eq!(j.get("message").and_then(|v| v.as_str()), Some(""));
    assert!(j.get("message").is_some());
}

#[test]
fn health_details_serialize_unicode_preserved() {
    let d = HealthDetails {
        status: HealthStatus::Healthy,
        latency_ms: 1,
        server_version: "バージョン".into(),
        message: Some("ok 🔥".into()),
    };
    let j = to_json(&d).unwrap();
    assert_eq!(
        j.get("server_version").and_then(|v| v.as_str()),
        Some("バージョン")
    );
    assert_eq!(j.get("message").and_then(|v| v.as_str()), Some("ok 🔥"));
}

#[test]
fn health_details_serialize_latency_zero() {
    let d = HealthDetails {
        status: HealthStatus::Healthy,
        latency_ms: 0,
        server_version: "8.0".into(),
        message: None,
    };
    let j = to_json(&d).unwrap();
    assert_eq!(j.get("latency_ms").and_then(Json::as_u64), Some(0));
}

#[test]
fn health_details_serialize_latency_u64_max() {
    let d = HealthDetails {
        status: HealthStatus::Healthy,
        latency_ms: u64::MAX,
        server_version: "8.0".into(),
        message: None,
    };
    let j = to_json(&d).unwrap();
    assert_eq!(j.get("latency_ms").and_then(Json::as_u64), Some(u64::MAX));
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
fn health_details_deserialize_null_message_is_none() {
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
fn health_details_deserialize_empty_message_is_some() {
    let d: HealthDetails = from_json(&json!({
        "status": "healthy",
        "latency_ms": 1,
        "server_version": "8.0",
        "message": ""
    }))
    .unwrap();
    assert_eq!(d.message.as_deref(), Some(""));
}

#[test]
fn health_details_deserialize_extra_fields_ignored() {
    let d: HealthDetails = from_json(&json!({
        "status": "healthy",
        "latency_ms": 1,
        "server_version": "8.0",
        "extra": "ignored",
        "another": 42,
        "nested": {"a": [1, 2]}
    }))
    .unwrap();
    assert_eq!(d.status, HealthStatus::Healthy);
}

#[test]
fn health_details_deserialize_latency_zero() {
    let d: HealthDetails = from_json(&json!({
        "status": "healthy",
        "latency_ms": 0,
        "server_version": "8.0"
    }))
    .unwrap();
    assert_eq!(d.latency_ms, 0);
}

#[test]
fn health_details_deserialize_latency_u64_max() {
    let d: HealthDetails = from_json(&json!({
        "status": "healthy",
        "latency_ms": u64::MAX,
        "server_version": "8.0"
    }))
    .unwrap();
    assert_eq!(d.latency_ms, u64::MAX);
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
fn health_details_rejects_string_latency() {
    let r = from_json::<HealthDetails>(&json!({
        "status": "healthy",
        "latency_ms": "1",
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
fn health_details_rejects_null_root() {
    let r = from_json::<HealthDetails>(&json!(null));
    assert!(r.is_err());
}

#[test]
fn health_details_rejects_array_root() {
    let r = from_json::<HealthDetails>(&json!([1, 2, 3]));
    assert!(r.is_err());
}

#[test]
fn health_details_clone_preserves_all_fields() {
    let a = HealthDetails {
        status: HealthStatus::Degraded,
        latency_ms: 100,
        server_version: "8.0".into(),
        message: Some("msg".into()),
    };
    let b = a.clone();
    assert_eq!(a.status, b.status);
    assert_eq!(a.latency_ms, b.latency_ms);
    assert_eq!(a.server_version, b.server_version);
    assert_eq!(a.message, b.message);
}

#[test]
fn health_details_debug_contains_type_name() {
    let d = HealthDetails {
        status: HealthStatus::Healthy,
        latency_ms: 1,
        server_version: "8.0".into(),
        message: None,
    };
    assert!(format!("{:?}", d).contains("HealthDetails"));
}

#[test]
fn health_details_roundtrip_full() {
    let original = HealthDetails {
        status: HealthStatus::Degraded,
        latency_ms: 100,
        server_version: "8.0.35".into(),
        message: Some("minor lag".into()),
    };
    let back: HealthDetails = from_json(&to_json(&original).unwrap()).unwrap();
    assert_eq!(back.status, original.status);
    assert_eq!(back.latency_ms, original.latency_ms);
    assert_eq!(back.server_version, original.server_version);
    assert_eq!(back.message, original.message);
}

#[test]
fn health_details_roundtrip_without_message() {
    let original = HealthDetails {
        status: HealthStatus::Healthy,
        latency_ms: 1,
        server_version: "8.0".into(),
        message: None,
    };
    let back: HealthDetails = from_json(&to_json(&original).unwrap()).unwrap();
    assert!(back.message.is_none());
}
