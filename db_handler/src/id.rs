use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::marker::PhantomData;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub struct Id<T> {
    value: i64,
    _marker: PhantomData<fn() -> T>,
}

impl<T> Id<T> {
    #[must_use]
    pub const fn new(value: i64) -> Self {
        Self {
            value,
            _marker: PhantomData,
        }
    }

    #[must_use]
    pub const fn value(self) -> i64 {
        self.value
    }
}

impl<T> Copy for Id<T> {}

#[allow(clippy::expl_impl_clone_on_copy)]
impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        *self
    }
}

#[allow(clippy::derivable_impls)]
impl<T> Default for Id<T> {
    fn default() -> Self {
        Self::new(0)
    }
}

impl<T> fmt::Debug for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Id").field(&self.value).finish()
    }
}

impl<T> fmt::Display for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.value, f)
    }
}

impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T> Eq for Id<T> {}

impl<T> PartialOrd for Id<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Id<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }
}

impl<T> Hash for Id<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl<T> Serialize for Id<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.value.serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for Id<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        i64::deserialize(deserializer).map(Self::new)
    }
}
