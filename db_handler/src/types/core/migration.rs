#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Migration {
    pub version: u64,
    pub name: String,
    pub up_sql: String,
    pub down_sql: Option<String>,
}

impl Migration {
    #[must_use]
    pub fn new(version: u64, name: impl Into<String>, up_sql: impl Into<String>) -> Self {
        Self {
            version,
            name: name.into(),
            up_sql: up_sql.into(),
            down_sql: None,
        }
    }

    #[must_use]
    pub fn with_down(mut self, down_sql: impl Into<String>) -> Self {
        self.down_sql = Some(down_sql.into());
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationState {
    Pending,
    Applied,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationStatus {
    pub migration: Migration,
    pub state: MigrationState,
    pub applied_at: Option<String>,
}

impl MigrationStatus {
    #[must_use]
    pub const fn new(
        migration: Migration,
        state: MigrationState,
        applied_at: Option<String>,
    ) -> Self {
        Self {
            migration,
            state,
            applied_at,
        }
    }
}
