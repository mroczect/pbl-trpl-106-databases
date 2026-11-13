#![allow(unused_crate_dependencies, clippy::unwrap_used, clippy::expect_used)]

use db_handler::constants::{MAX_PAGE_NUMBER, MAX_PAGE_SIZE};
use db_handler::types::core::PageRequest;
use db_handler::validation::validate_pagination;
use db_handler::DbError;

#[test]
fn accepts_default() {
    assert!(validate_pagination(&PageRequest::default()).is_ok());
}

#[test]
fn accepts_first_page() {
    assert!(validate_pagination(&PageRequest { page: 1, per_page: 1 }).is_ok());
}

#[test]
fn accepts_max_boundary() {
    assert!(validate_pagination(&PageRequest {
        page: MAX_PAGE_NUMBER,
        per_page: MAX_PAGE_SIZE,
    })
    .is_ok());
}

#[test]
fn rejects_page_zero() {
    let r = validate_pagination(&PageRequest { page: 0, per_page: 20 });
    assert!(matches!(r, Err(DbError::Validation { .. })));
}

#[test]
fn rejects_page_above_max() {
    let r = validate_pagination(&PageRequest {
        page: MAX_PAGE_NUMBER + 1,
        per_page: 20,
    });
    assert!(matches!(r, Err(DbError::Validation { .. })));
}

#[test]
fn rejects_per_page_zero() {
    let r = validate_pagination(&PageRequest { page: 1, per_page: 0 });
    assert!(matches!(r, Err(DbError::Validation { .. })));
}

#[test]
fn rejects_per_page_above_max() {
    let r = validate_pagination(&PageRequest {
        page: 1,
        per_page: MAX_PAGE_SIZE + 1,
    });
    assert!(matches!(r, Err(DbError::Validation { .. })));
}

#[test]
fn page_checked_before_per_page() {
    let err = validate_pagination(&PageRequest { page: 0, per_page: 0 }).unwrap_err();
    if let DbError::Validation { message } = err {
        assert!(message.contains("page"));
    }
}

#[test]
fn error_message_includes_page_numbers() {
    let err = validate_pagination(&PageRequest { page: 0, per_page: 20 }).unwrap_err();
    if let DbError::Validation { message } = err {
    	assert!(message.contains('0'));
        assert!(message.contains(&MAX_PAGE_NUMBER.to_string()));
    }
}

#[test]
fn error_message_includes_per_page_numbers() {
    let err = validate_pagination(&PageRequest {
        page: 1,
        per_page: MAX_PAGE_SIZE + 1,
    })
    .unwrap_err();
    if let DbError::Validation { message } = err {
        assert!(message.contains(&(MAX_PAGE_SIZE + 1).to_string()));
        assert!(message.contains(&MAX_PAGE_SIZE.to_string()));
    }
}
