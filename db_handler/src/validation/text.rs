use crate::constants::MAX_STRING_FIELD_LENGTH;
use crate::{DbError, Result};

pub fn validate_not_empty(field: &str, value: &str) -> Result<()> {
    if value.is_empty() {
        return Err(DbError::Validation {
            message: format!("{field} must not be empty"),
        });
    }
    Ok(())
}

pub fn validate_text(field: &str, value: &str) -> Result<()> {
    validate_text_len(field, value, MAX_STRING_FIELD_LENGTH)
}

pub fn validate_text_len(field: &str, value: &str, max_len: usize) -> Result<()> {
    if value.len() > max_len {
        return Err(DbError::Validation {
            message: format!("{field} exceeds {max_len} bytes (got {})", value.len()),
        });
    }
    if value
        .bytes()
        .any(|b| b.is_ascii_control() && b != b'\n' && b != b'\t')
    {
        return Err(DbError::Validation {
            message: format!("{field} contains control characters"),
        });
    }
    Ok(())
}
