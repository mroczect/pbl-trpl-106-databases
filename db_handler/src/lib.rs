pub mod config;
pub mod constants;
pub mod error;
pub mod json;
pub mod prelude;
pub mod traits;
pub mod types;
pub mod validation;

pub use config::PoolConfig;
pub use error::{DbError, Result};
pub use json::Json;

pub use traits::core::{
    Connection, DatabaseInfoProvider, Executor, HealthCheck, MigrationRunner, SchemaManager,
    Transaction, TransactionManager,
};
pub use traits::repository::CrudRepository;

pub use types::core::{
    Column, ColumnDef, ColumnValue, Condition, Constraint, ConstraintKind, DataType, DatabaseInfo,
    DeleteInput, ExecuteResult, Filter, HealthDetails, HealthStatus, Index, InsertInput,
    IsolationLevel, Migration, MigrationState, MigrationStatus, PageMeta, PageRequest, Paginated,
    Query, QueryResult, Row, SortDirection, Table, TableChange, UpdateInput, Value,
};
