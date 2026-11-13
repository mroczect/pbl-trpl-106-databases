use crate::constants::{MAX_PAGE_NUMBER, MAX_PAGE_SIZE};
use crate::types::core::PageRequest;
use crate::{DbError, Result};

pub fn validate_pagination(req: &PageRequest) -> Result<()> {
    if req.page == 0 || req.page > MAX_PAGE_NUMBER {
        return Err(DbError::Validation {
            message: format!(
                "page must be between 1 and {MAX_PAGE_NUMBER}, got {}",
                req.page
            ),
        });
    }
    if req.per_page == 0 || req.per_page > MAX_PAGE_SIZE {
        return Err(DbError::Validation {
            message: format!(
                "per_page must be between 1 and {MAX_PAGE_SIZE}, got {}",
                req.per_page
            ),
        });
    }
    Ok(())
}
