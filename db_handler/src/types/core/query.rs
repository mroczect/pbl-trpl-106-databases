use crate::types::core::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct Query {
    pub sql: String,
    pub params: Vec<Value>,
}

impl Query {
    #[must_use]
    pub fn new(sql: impl Into<String>) -> Self {
        Self {
            sql: sql.into(),
            params: Vec::new(),
        }
    }

    #[must_use]
    pub fn bind(mut self, value: impl Into<Value>) -> Self {
        self.params.push(value.into());
        self
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.sql.is_empty()
    }

    #[must_use]
    pub fn param_count(&self) -> usize {
        self.params.len()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColumnValue {
    pub column: String,
    pub value: Value,
}

impl ColumnValue {
    #[must_use]
    pub fn new(column: impl Into<String>, value: impl Into<Value>) -> Self {
        Self {
            column: column.into(),
            value: value.into(),
        }
    }
}
