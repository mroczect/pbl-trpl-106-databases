use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[non_exhaustive]
pub enum DbError {
    #[error("missing environment variable: {name}")]
    MissingEnv { name: String },

    #[error("invalid environment variable {name}: {reason}")]
    InvalidEnv { name: String, reason: String },

    #[error("connection error: {message}")]
    Connection { message: String },

    #[error("query error: {message}")]
    Query { message: String },

    #[error("database error: {message}")]
    Database { message: String },

    #[error("resource not found")]
    NotFound,

    #[error("validation failed: {message}")]
    Validation { message: String },

    #[error("json error: {message}")]
    Json { message: String },

    #[error("io error: {message}")]
    Io { message: String },
}

pub type Result<T> = std::result::Result<T, DbError>;

impl From<serde_json::Error> for DbError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json {
            message: e.to_string(),
        }
    }
}
