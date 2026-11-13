use serde::{Deserialize, Serialize};

use crate::constants::DEFAULT_PAGE_SIZE;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PageRequest {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_per_page")]
    pub per_page: u64,
}

const fn default_page() -> u64 {
    1
}
const fn default_per_page() -> u64 {
    DEFAULT_PAGE_SIZE
}

impl Default for PageRequest {
    fn default() -> Self {
        Self {
            page: 1,
            per_page: DEFAULT_PAGE_SIZE,
        }
    }
}

impl PageRequest {
        #[must_use]
    pub const fn offset(&self) -> u64 {
        self.page.saturating_sub(1).saturating_mul(self.per_page)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PageMeta {
    pub page: u64,
    pub per_page: u64,
    pub total: u64,
    pub total_pages: u64,
}

impl PageMeta {
        #[must_use]
    pub const fn new(page: u64, per_page: u64, total: u64) -> Self {
        let total_pages = if per_page == 0 {
            0
        } else {
            total.div_ceil(per_page)
        };
        Self {
            page,
            per_page,
            total,
            total_pages,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paginated<T> {
    pub items: Vec<T>,
    pub meta: PageMeta,
}
