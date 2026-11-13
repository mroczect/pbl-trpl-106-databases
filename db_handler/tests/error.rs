#![allow(unused_crate_dependencies, clippy::unwrap_used, clippy::expect_used)]

use std::collections::HashSet;
use std::error::Error;

use db_handler::json::{from_json, to_json};
use db_handler::types::core::SortDirection;
use db_handler::DbError;
use serde_json::json;


const EXPECTED_VARIANT_COUNT: usize = 9;

fn all_variants() -> Vec<(&'static str, DbError)> {
    vec![
        (
            "missing_env",
            DbError::MissingEnv {
                name: "DATABASE_URL".into(),
            },
        ),
        (
            "invalid_env",
            DbError::InvalidEnv {
                name: "DATABASE_MAX".into(),
                reason: "must be > 0".into(),
            },
        ),
        (
            "connection",
            DbError::Connection {
                message: "refused".into(),
            },
        ),
        (
            "query",
            DbError::Query {
                message: "syntax error".into(),
            },
        ),
        (
            "database",
            DbError::Database {
                message: "constraint violation".into(),
            },
        ),
        ("not_found", DbError::NotFound),
        (
            "validation",
            DbError::Validation {
                message: "bad input".into(),
            },
        ),
        (
            "json",
            DbError::Json {
                message: "parse failed".into(),
            },
        ),
        (
            "io",
            DbError::Io {
                message: "socket closed".into(),
            },
        ),
    ]
}

#[derive(serde::Deserialize, Debug)]
struct RequiredField {
    name: String,
}

#[test]
fn required_field_deserializes_when_present() {
    let v: RequiredField = from_json(&json!({"name": "Alice"})).unwrap();
    assert_eq!(v.name, "Alice");
}

#[test]
fn serialize_missing_env_exact_shape() {
    let e = DbError::MissingEnv {
        name: "DATABASE_URL".into(),
    };
    assert_eq!(
        to_json(&e).unwrap(),
        json!({"kind": "missing_env", "name": "DATABASE_URL"})
    );
}

#[test]
fn serialize_invalid_env_exact_shape() {
    let e = DbError::InvalidEnv {
        name: "DATABASE_MAX".into(),
        reason: "must be > 0".into(),
    };
    assert_eq!(
        to_json(&e).unwrap(),
        json!({
            "kind": "invalid_env",
            "name": "DATABASE_MAX",
            "reason": "must be > 0"
        })
    );
}

#[test]
fn serialize_connection_exact_shape() {
    let e = DbError::Connection {
        message: "refused".into(),
    };
    assert_eq!(
        to_json(&e).unwrap(),
        json!({"kind": "connection", "message": "refused"})
    );
}

#[test]
fn serialize_query_exact_shape() {
    let e = DbError::Query {
        message: "syntax".into(),
    };
    assert_eq!(
        to_json(&e).unwrap(),
        json!({"kind": "query", "message": "syntax"})
    );
}

#[test]
fn serialize_database_exact_shape() {
    let e = DbError::Database {
        message: "constraint".into(),
    };
    assert_eq!(
        to_json(&e).unwrap(),
        json!({"kind": "database", "message": "constraint"})
    );
}

#[test]
fn serialize_not_found_exact_shape() {
    let e = DbError::NotFound;
    assert_eq!(to_json(&e).unwrap(), json!({"kind": "not_found"}));
}

#[test]
fn serialize_validation_exact_shape() {
    let e = DbError::Validation {
        message: "bad".into(),
    };
    assert_eq!(
        to_json(&e).unwrap(),
        json!({"kind": "validation", "message": "bad"})
    );
}

#[test]
fn serialize_json_exact_shape() {
    let e = DbError::Json {
        message: "parse".into(),
    };
    assert_eq!(
        to_json(&e).unwrap(),
        json!({"kind": "json", "message": "parse"})
    );
}

#[test]
fn serialize_io_exact_shape() {
    let e = DbError::Io {
        message: "closed".into(),
    };
    assert_eq!(
        to_json(&e).unwrap(),
        json!({"kind": "io", "message": "closed"})
    );
}

#[test]
fn not_found_serializes_without_message_field() {
    let json = to_json(&DbError::NotFound).unwrap();
    let obj = json.as_object().unwrap();
    assert_eq!(obj.len(), 1);
    assert!(obj.contains_key("kind"));
    assert!(!obj.contains_key("message"));
}


#[test]
fn display_missing_env() {
    let e = DbError::MissingEnv { name: "X".into() };
    assert_eq!(e.to_string(), "missing environment variable: X");
}

#[test]
fn display_invalid_env() {
    let e = DbError::InvalidEnv {
        name: "X".into(),
        reason: "Y".into(),
    };
    assert_eq!(e.to_string(), "invalid environment variable X: Y");
}

#[test]
fn display_connection() {
    let e = DbError::Connection {
        message: "refused".into(),
    };
    assert_eq!(e.to_string(), "connection error: refused");
}

#[test]
fn display_query() {
    let e = DbError::Query {
        message: "syntax".into(),
    };
    assert_eq!(e.to_string(), "query error: syntax");
}

#[test]
fn display_database() {
    let e = DbError::Database {
        message: "constraint".into(),
    };
    assert_eq!(e.to_string(), "database error: constraint");
}

#[test]
fn display_not_found() {
    assert_eq!(DbError::NotFound.to_string(), "resource not found");
}

#[test]
fn display_validation() {
    let e = DbError::Validation {
        message: "bad".into(),
    };
    assert_eq!(e.to_string(), "validation failed: bad");
}

#[test]
fn display_json() {
    let e = DbError::Json {
        message: "boom".into(),
    };
    assert_eq!(e.to_string(), "json error: boom");
}

#[test]
fn display_io() {
    let e = DbError::Io {
        message: "closed".into(),
    };
    assert_eq!(e.to_string(), "io error: closed");
}

#[test]
fn not_found_display_has_no_trailing_separator() {
    let s = DbError::NotFound.to_string();
    assert!(!s.ends_with(':'));
    assert!(!s.ends_with(": "));
}


#[test]
fn debug_each_variant_contains_name() {
    let cases: [(&str, DbError); 9] = [
        ("MissingEnv", DbError::MissingEnv { name: "x".into() }),
        ("InvalidEnv", DbError::InvalidEnv { name: "x".into(), reason: "y".into() }),
        ("Connection", DbError::Connection { message: "m".into() }),
        ("Query", DbError::Query { message: "m".into() }),
        ("Database", DbError::Database { message: "m".into() }),
        ("NotFound", DbError::NotFound),
        ("Validation", DbError::Validation { message: "m".into() }),
        ("Json", DbError::Json { message: "m".into() }),
        ("Io", DbError::Io { message: "m".into() }),
    ];
    for (name, variant) in cases {
        let dbg = format!("{variant:?}");
        assert!(dbg.contains(name), "debug of {name} was {dbg}");
    }
}

#[test]
fn debug_preserves_message_content() {
    let e = DbError::Connection {
        message: "very specific text".into(),
    };
    assert!(format!("{e:?}").contains("very specific text"));
}


#[test]
fn display_differs_from_debug_for_all_variants() {
    for (kind, variant) in all_variants() {
        let dbg = format!("{variant:?}");
        let disp = variant.to_string();
        assert_ne!(dbg, disp, "Display equals Debug for {kind}");
    }
}

#[test]
fn display_differs_from_json_for_all_variants() {
    for (kind, variant) in all_variants() {
        let json_str = to_json(&variant).unwrap().to_string();
        let disp = variant.to_string();
        assert_ne!(disp, json_str, "Display equals JSON for {kind}");
    }
}

#[test]
fn debug_differs_from_json_for_all_variants() {
    for (kind, variant) in all_variants() {
        let json_str = to_json(&variant).unwrap().to_string();
        let dbg = format!("{variant:?}");
        assert_ne!(dbg, json_str, "Debug equals JSON for {kind}");
    }
}


#[test]
fn all_kinds_are_unique() {
    let mut seen = HashSet::new();
    for (_, variant) in all_variants() {
        let kind = to_json(&variant).unwrap()["kind"]
            .as_str()
            .expect("kind must be a string")
            .to_string();
        assert!(seen.insert(kind.clone()), "duplicate kind: {kind}");
    }
    assert_eq!(seen.len(), EXPECTED_VARIANT_COUNT);
}

#[test]
fn all_kinds_are_snake_case() {
    for (_, variant) in all_variants() {
        let kind = to_json(&variant).unwrap()["kind"]
            .as_str()
            .expect("kind must be a string")
            .to_string();
        assert!(!kind.is_empty(), "kind must not be empty");
        let first = kind.chars().next().unwrap();
        assert!(
            first.is_ascii_lowercase(),
            "kind {kind} must start lowercase"
        );
        for c in kind.chars() {
            assert!(
                c.is_ascii_lowercase() || c == '_' || c.is_ascii_digit(),
                "kind {kind} contains forbidden char {c:?}"
            );
        }
    }
}

#[test]
fn kind_field_matches_expected_name() {
    for (expected_kind, variant) in all_variants() {
        let actual = to_json(&variant).unwrap()["kind"]
            .as_str()
            .expect("kind must be a string")
            .to_string();
        assert_eq!(actual, expected_kind, "wrong kind for {variant:?}");
    }
}

#[test]
fn every_variant_serializes_as_object_with_kind() {
    for (kind, variant) in all_variants() {
        let json = to_json(&variant).unwrap();
        let obj = json
            .as_object()
            .unwrap_or_else(|| panic!("{kind} did not serialize to object"));
        assert!(obj.contains_key("kind"), "{kind} missing kind field");
        assert!(json["kind"].is_string(), "{kind} kind not a string");
    }
}


#[test]
fn serialize_message_with_double_quotes() {
    let e = DbError::Connection {
        message: "he said \"hello\"".into(),
    };
    let json = to_json(&e).unwrap();
    assert_eq!(json["message"].as_str(), Some("he said \"hello\""));
}

#[test]
fn serialize_message_with_backslash() {
    let e = DbError::Connection {
        message: "C:\\path\\to\\file".into(),
    };
    assert_eq!(
        to_json(&e).unwrap()["message"].as_str(),
        Some("C:\\path\\to\\file")
    );
}

#[test]
fn serialize_message_with_newline_and_tab() {
    let e = DbError::Connection {
        message: "line1\nline2\tcol".into(),
    };
    assert_eq!(
        to_json(&e).unwrap()["message"].as_str(),
        Some("line1\nline2\tcol")
    );
}

#[test]
fn serialize_message_with_nul_byte() {
    let e = DbError::Connection {
        message: "before\0after".into(),
    };
    assert_eq!(
        to_json(&e).unwrap()["message"].as_str(),
        Some("before\0after")
    );
}

#[test]
fn serialize_message_with_unicode() {
    let e = DbError::InvalidEnv {
        name: "变量名".into(),
        reason: "必须大于零".into(),
    };
    let json = to_json(&e).unwrap();
    assert_eq!(json["name"].as_str(), Some("变量名"));
    assert_eq!(json["reason"].as_str(), Some("必须大于零"));
}

#[test]
fn serialize_message_with_emoji() {
    let e = DbError::Connection {
        message: "🔥 boom".into(),
    };
    assert_eq!(to_json(&e).unwrap()["message"].as_str(), Some("🔥 boom"));
}

#[test]
fn display_passes_through_special_chars_verbatim() {
    let e = DbError::Connection {
        message: "quoted \" text".into(),
    };
    assert_eq!(e.to_string(), "connection error: quoted \" text");
}


#[test]
fn serialize_empty_message() {
    let e = DbError::Validation {
        message: String::new(),
    };
    assert_eq!(
        to_json(&e).unwrap(),
        json!({"kind": "validation", "message": ""})
    );
}

#[test]
fn display_empty_message() {
    let e = DbError::Validation {
        message: String::new(),
    };
    assert_eq!(e.to_string(), "validation failed: ");
}

#[test]
fn serialize_very_long_message() {
    let long = "a".repeat(100_000);
    let e = DbError::Io {
        message: long.clone(),
    };
    let json = to_json(&e).unwrap();
    assert_eq!(json["message"].as_str().unwrap().len(), 100_000);
}

#[test]
fn serialize_empty_env_name() {
    let e = DbError::MissingEnv { name: String::new() };
    assert_eq!(
        to_json(&e).unwrap(),
        json!({"kind": "missing_env", "name": ""})
    );
}


#[test]
fn from_serde_error_invalid_type() {
    let err = from_json::<u32>(&json!("text")).unwrap_err();
    assert!(matches!(err, DbError::Json { .. }));
    if let DbError::Json { message } = err {
        assert!(!message.is_empty());
        assert!(message.contains("invalid type"), "message was: {message}");
    }
}

#[test]
fn from_serde_error_missing_field() {
    let err = from_json::<RequiredField>(&json!({})).unwrap_err();
    if let DbError::Json { message } = err {
        assert!(message.contains("missing field"), "message was: {message}");
        assert!(message.contains("name"));
    } else {
        panic!("expected DbError::Json");
    }
}

#[test]
fn from_serde_error_unknown_variant() {
    let err = from_json::<SortDirection>(&json!("sideways")).unwrap_err();
    if let DbError::Json { message } = err {
        assert!(message.contains("unknown variant"), "message was: {message}");
        assert!(message.contains("sideways"));
    } else {
        panic!("expected DbError::Json");
    }
}

#[test]
fn from_serde_error_integer_out_of_range() {
    let err = from_json::<u8>(&json!(256)).unwrap_err();
    if let DbError::Json { message } = err {
        assert!(
            message.contains("256") || message.contains("u8"),
            "message was: {message}"
        );
    } else {
        panic!("expected DbError::Json");
    }
}

#[test]
fn from_serde_error_on_null_when_object_expected() {
    let err = from_json::<RequiredField>(&json!(null)).unwrap_err();
    assert!(matches!(err, DbError::Json { .. }));
}


#[test]
fn implements_std_error_trait() {
    fn assert_error<E: Error>() {}
    assert_error::<DbError>();
}

#[test]
fn usable_as_boxed_dyn_error() {
    let e = DbError::NotFound;
    let boxed: Box<dyn Error> = Box::new(e);
    assert!(boxed.to_string().contains("not found"));
}

#[test]
fn propagates_through_question_mark_into_boxed_error() {
    fn helper() -> Result<(), Box<dyn Error>> {
        Err(DbError::NotFound)?;
        Ok(())
    }
    let err = helper().unwrap_err();
    assert!(err.to_string().contains("not found"));
}

#[test]
fn source_is_none_for_all_variants() {
    // The thiserror derive does not apply #[source] to any variant, and the
    // From<serde_json::Error> impl is manual (not #[from]), so no variant
    for (kind, variant) in all_variants() {
        assert!(
            variant.source().is_none(),
            "{kind} unexpectedly has a source"
        );
    }
}


#[test]
fn identical_errors_serialize_identically() {
    let a = DbError::Connection {
        message: "same".into(),
    };
    let b = DbError::Connection {
        message: "same".into(),
    };
    assert_eq!(to_json(&a).unwrap(), to_json(&b).unwrap());
}

#[test]
fn different_messages_serialize_differently() {
    let a = DbError::Connection {
        message: "one".into(),
    };
    let b = DbError::Connection {
        message: "two".into(),
    };
    assert_ne!(to_json(&a).unwrap(), to_json(&b).unwrap());
}

#[test]
fn different_variants_with_same_message_serialize_differently() {
    let a = DbError::Connection {
        message: "not found".into(),
    };
    let b = DbError::NotFound;
    assert_ne!(to_json(&a).unwrap(), to_json(&b).unwrap());
}

#[test]
fn serialization_is_deterministic_across_repeated_calls() {
    let e = DbError::InvalidEnv {
        name: "X".into(),
        reason: "Y".into(),
    };
    let first = to_json(&e).unwrap();
    for _ in 0..10 {
        assert_eq!(to_json(&e).unwrap(), first);
    }
}


#[test]
fn expected_variant_count_is_up_to_date() {
    assert_eq!(all_variants().len(), EXPECTED_VARIANT_COUNT);
}
