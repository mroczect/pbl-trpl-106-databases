#![allow(unused_crate_dependencies, clippy::unwrap_used, clippy::expect_used)]

use db_handler::constants::MAX_STRING_FIELD_LENGTH;
use db_handler::validation::{validate_not_empty, validate_text, validate_text_len};
use db_handler::DbError;

// ===== validate_not_empty =====

#[test]
fn validate_not_empty_rejects_empty() {
    let r = validate_not_empty("name", "");
    assert!(matches!(r, Err(DbError::Validation { .. })));
}

#[test]
fn validate_not_empty_accepts_single_char() {
    assert!(validate_not_empty("name", "a").is_ok());
}

#[test]
fn validate_not_empty_accepts_whitespace_only() {
    assert!(validate_not_empty("name", "   ").is_ok());
}

#[test]
fn validate_not_empty_accepts_unicode() {
    assert!(validate_not_empty("name", "日本語").is_ok());
}

#[test]
fn validate_not_empty_error_mentions_field() {
    let err = validate_not_empty("username", "").unwrap_err();
    if let DbError::Validation { message } = err {
        assert!(message.contains("username"));
    }
}

#[test]
fn validate_text_accepts_empty() {
    assert!(validate_text("bio", "").is_ok());
}

#[test]
fn validate_text_accepts_short_string() {
    assert!(validate_text("bio", "hello world").is_ok());
}

#[test]
fn validate_text_accepts_newline() {
    assert!(validate_text("bio", "line1\nline2").is_ok());
}

#[test]
fn validate_text_accepts_tab() {
    assert!(validate_text("bio", "col1\tcol2").is_ok());
}

#[test]
fn validate_text_rejects_nul() {
    let r = validate_text("bio", "a\0b");
    assert!(matches!(r, Err(DbError::Validation { .. })));
}

#[test]
fn validate_text_rejects_carriage_return() {
    let r = validate_text("bio", "a\rb");
    assert!(matches!(r, Err(DbError::Validation { .. })));
}

#[test]
fn validate_text_rejects_del_char() {
    let r = validate_text("bio", "a\u{7F}b");
    assert!(matches!(r, Err(DbError::Validation { .. })));
}

#[test]
fn validate_text_rejects_byte_0x01() {
    let r = validate_text("bio", "a\u{0001}b");
    assert!(matches!(r, Err(DbError::Validation { .. })));
}

#[test]
fn validate_text_accepts_string_at_max_length() {
    let s = "a".repeat(MAX_STRING_FIELD_LENGTH);
    assert!(validate_text("bio", &s).is_ok());
}

#[test]
fn validate_text_rejects_string_over_max_length() {
    let s = "a".repeat(MAX_STRING_FIELD_LENGTH + 1);
    assert!(validate_text("bio", &s).is_err());
}

#[test]
fn validate_text_length_is_bytes_not_chars() {
    let s = "éé";
    assert_eq!(s.len(), 4);
    assert!(validate_text_len("bio", s, 4).is_ok());
    assert!(validate_text_len("bio", s, 3).is_err());
}

#[test]
fn validate_text_error_mentions_field_name() {
    let err = validate_text("description", "a\u{0001}b").unwrap_err();
    if let DbError::Validation { message } = err {
        assert!(message.contains("description"));
    }
}

#[test]
fn validate_text_len_custom_max() {
    assert!(validate_text_len("x", "12345", 5).is_ok());
    assert!(validate_text_len("x", "123456", 5).is_err());
}

#[test]
fn validate_text_len_zero_max() {
    assert!(validate_text_len("x", "", 0).is_ok());
    assert!(validate_text_len("x", "a", 0).is_err());
}

#[test]
fn validate_text_len_message_mentions_actual_length() {
    let err = validate_text_len("x", "abcdef", 3).unwrap_err();
    if let DbError::Validation { message } = err {
        assert!(message.contains('6'));
        assert!(message.contains('3'));
    }
}
