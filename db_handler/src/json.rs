use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::{DbError, Result};

pub use serde_json::{Map, Number, Value as Json};

#[must_use]
pub fn empty_object() -> Json {
    Json::Object(Map::new())
}

pub fn to_json<T: Serialize>(value: &T) -> Result<Json> {
    serde_json::to_value(value).map_err(DbError::from)
}

pub fn from_json<T: DeserializeOwned>(json: &Json) -> Result<T> {
    serde_json::from_value(json.clone()).map_err(DbError::from)
}

pub fn expect_object(json: &Json) -> Result<&Map<String, Json>> {
    json.as_object().ok_or_else(|| DbError::Validation {
        message: "expected JSON object".into(),
    })
}

pub fn require_field<'a>(obj: &'a Map<String, Json>, name: &str) -> Result<&'a Json> {
    obj.get(name).ok_or_else(|| DbError::Validation {
        message: format!("missing required field: {name}"),
    })
}
