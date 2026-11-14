use crate::constants::MAX_QUERY_LENGTH;
use crate::{DbError, Result};

const MAX_IDENTIFIER_LEN: usize = 64;

pub fn validate_identifier(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(DbError::Validation {
            message: "identifier must not be empty".into(),
        });
    }

    if name.len() > MAX_IDENTIFIER_LEN {
        return Err(DbError::Validation {
            message: format!(
                "identifier exceeds {MAX_IDENTIFIER_LEN} chars (got {})",
                name.len()
            ),
        });
    }

    if name.len() > MAX_QUERY_LENGTH {
        return Err(DbError::Validation {
            message: "identifier exceeds MAX_QUERY_LENGTH".into(),
        });
    }

    let mut chars = name.chars();
    let first = chars.next().ok_or_else(|| DbError::Validation {
        message: "identifier must not be empty".into(),
    })?;

    if !(first.is_ascii_alphabetic() || first == '_') {
        return Err(DbError::Validation {
            message: format!("identifier must start with letter or underscore, got {first:?}"),
        });
    }

    for c in chars {
        if !(c.is_ascii_alphanumeric() || c == '_') {
            return Err(DbError::Validation {
                message: format!("identifier contains invalid char {c:?}"),
            });
        }
    }

    Ok(())
}
