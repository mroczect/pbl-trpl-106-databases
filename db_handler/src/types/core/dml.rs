use crate::types::core::{ColumnValue, Value};

#[derive(Debug, Clone, PartialEq)]
pub enum Condition {
    Eq { column: String, value: Value },
    Ne { column: String, value: Value },
    Lt { column: String, value: Value },
    Le { column: String, value: Value },
    Gt { column: String, value: Value },
    Ge { column: String, value: Value },
    In { column: String, values: Vec<Value> },
    IsNull { column: String },
    IsNotNull { column: String },
    And(Vec<Self>),
    Or(Vec<Self>),
}

impl Condition {
    #[must_use]
    pub fn eq(column: impl Into<String>, value: impl Into<Value>) -> Self {
        Self::Eq {
            column: column.into(),
            value: value.into(),
        }
    }

    #[must_use]
    pub fn is_null(column: impl Into<String>) -> Self {
        Self::IsNull {
            column: column.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Filter {
    pub conditions: Vec<Condition>,
}

impl Filter {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    #[allow(clippy::should_implement_trait)]
    pub fn add(mut self, cond: Condition) -> Self {
        self.conditions.push(cond);
        self
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.conditions.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InsertInput {
    pub table: String,
    pub values: Vec<ColumnValue>,
}

impl InsertInput {
    #[must_use]
    pub fn new(table: impl Into<String>) -> Self {
        Self {
            table: table.into(),
            values: Vec::new(),
        }
    }

    #[must_use]
    pub fn set(mut self, column: impl Into<String>, value: impl Into<Value>) -> Self {
        self.values.push(ColumnValue::new(column, value));
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpdateInput {
    pub table: String,
    pub set: Vec<ColumnValue>,
    pub filter: Filter,
}

impl UpdateInput {
    #[must_use]
    pub fn new(table: impl Into<String>) -> Self {
        Self {
            table: table.into(),
            set: Vec::new(),
            filter: Filter::new(),
        }
    }

    #[must_use]
    pub fn set(mut self, column: impl Into<String>, value: impl Into<Value>) -> Self {
        self.set.push(ColumnValue::new(column, value));
        self
    }

    #[must_use]
    pub fn where_(mut self, filter: Filter) -> Self {
        self.filter = filter;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeleteInput {
    pub table: String,
    pub filter: Filter,
}

impl DeleteInput {
    #[must_use]
    pub fn new(table: impl Into<String>) -> Self {
        Self {
            table: table.into(),
            filter: Filter::new(),
        }
    }

    #[must_use]
    pub fn where_(mut self, filter: Filter) -> Self {
        self.filter = filter;
        self
    }
}
