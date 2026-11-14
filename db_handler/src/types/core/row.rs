use serde::{Serialize, Serializer};

use crate::{DbError, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Uint(u64),
    Float(f64),
    String(String),
    Bytes(Vec<u8>),
    Timestamp(String),
}

impl Value {
    #[must_use]
    pub const fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) | Self::Timestamp(s) => Some(s),
            Self::Null
            | Self::Bool(_)
            | Self::Int(_)
            | Self::Uint(_)
            | Self::Float(_)
            | Self::Bytes(_) => None,
        }
    }

    #[must_use]
    pub const fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Int(i) => Some(*i),
            Self::Null
            | Self::Bool(_)
            | Self::Uint(_)
            | Self::Float(_)
            | Self::String(_)
            | Self::Bytes(_)
            | Self::Timestamp(_) => None,
        }
    }

    #[must_use]
    pub const fn as_u64(&self) -> Option<u64> {
        match self {
            Self::Uint(u) => Some(*u),
            Self::Null
            | Self::Bool(_)
            | Self::Int(_)
            | Self::Float(_)
            | Self::String(_)
            | Self::Bytes(_)
            | Self::Timestamp(_) => None,
        }
    }

    #[must_use]
    pub const fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            Self::Null
            | Self::Int(_)
            | Self::Uint(_)
            | Self::Float(_)
            | Self::String(_)
            | Self::Bytes(_)
            | Self::Timestamp(_) => None,
        }
    }

    #[must_use]
    pub const fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Float(f) => Some(*f),
            Self::Null
            | Self::Bool(_)
            | Self::Int(_)
            | Self::Uint(_)
            | Self::String(_)
            | Self::Bytes(_)
            | Self::Timestamp(_) => None,
        }
    }

    #[must_use]
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Self::Bytes(b) => Some(b),
            Self::Null
            | Self::Bool(_)
            | Self::Int(_)
            | Self::Uint(_)
            | Self::Float(_)
            | Self::String(_)
            | Self::Timestamp(_) => None,
        }
    }
}

macro_rules! impl_from_identity {
    ($t:ty, $variant:ident) => {
        impl From<$t> for Value {
            fn from(v: $t) -> Self {
                Self::$variant(v)
            }
        }
    };
}

macro_rules! impl_from_convert {
    ($t:ty, $variant:ident, $conv:path) => {
        impl From<$t> for Value {
            fn from(v: $t) -> Self {
                Self::$variant($conv(v))
            }
        }
    };
}

impl_from_identity!(bool, Bool);
impl_from_identity!(i64, Int);
impl_from_identity!(u64, Uint);
impl_from_identity!(f64, Float);
impl_from_identity!(String, String);
impl_from_identity!(Vec<u8>, Bytes);

impl_from_convert!(i8, Int, i64::from);
impl_from_convert!(i16, Int, i64::from);
impl_from_convert!(i32, Int, i64::from);

impl_from_convert!(u8, Uint, u64::from);
impl_from_convert!(u16, Uint, u64::from);
impl_from_convert!(u32, Uint, u64::from);

impl_from_convert!(f32, Float, f64::from);

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Self::String(v.to_string())
    }
}

impl<T: Into<Self>> From<Option<T>> for Value {
    fn from(v: Option<T>) -> Self {
        v.map_or(Self::Null, Into::into)
    }
}

impl Serialize for Value {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        match self {
            Self::Null => s.serialize_none(),
            Self::Bool(b) => s.serialize_bool(*b),
            Self::Int(i) => s.serialize_i64(*i),
            Self::Uint(u) => s.serialize_u64(*u),
            Self::Float(f) => s.serialize_f64(*f),
            Self::String(v) => s.serialize_str(v),
            Self::Bytes(b) => s.serialize_bytes(b),
            Self::Timestamp(t) => s.serialize_str(t),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Column {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
}

impl Column {
    #[must_use]
    pub fn new(name: impl Into<String>, data_type: impl Into<String>, nullable: bool) -> Self {
        Self {
            name: name.into(),
            data_type: data_type.into(),
            nullable,
        }
    }
}

pub trait Row: Send + Sync + std::fmt::Debug {
    fn get(&self, column: &str) -> Result<&Value>;
    fn columns(&self) -> &[Column];
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;

    fn try_get(&self, column: &str) -> Option<&Value> {
        self.get(column).ok()
    }
}

#[must_use]
pub fn missing_column(column: &str) -> DbError {
    DbError::Validation {
        message: format!("column not found: {column}"),
    }
}
