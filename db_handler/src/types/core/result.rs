use crate::types::core::{Column, Row};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ExecuteResult {
    pub rows_affected: u64,
    pub last_insert_id: Option<u64>,
}

impl ExecuteResult {
    #[must_use]
    pub const fn new(rows_affected: u64, last_insert_id: Option<u64>) -> Self {
        Self {
            rows_affected,
            last_insert_id,
        }
    }
}

#[derive(Debug)]
pub struct QueryResult {
    pub rows: Vec<Box<dyn Row>>,
    pub columns: Vec<Column>,
}

impl QueryResult {
    #[must_use]
    pub fn new(rows: Vec<Box<dyn Row>>, columns: Vec<Column>) -> Self {
        Self { rows, columns }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}
