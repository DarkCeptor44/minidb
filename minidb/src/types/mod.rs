use std::fmt::{Debug, Display};

use cuid2::slug;
use serde::{Deserialize, Serialize};

/// Represents a CUID2 slug
#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Id(Option<String>);

impl<S> From<S> for Id
where
    S: AsRef<str>,
{
    fn from(value: S) -> Self {
        let value = value.as_ref().trim();
        if value.is_empty() {
            Id::default()
        } else {
            Id::with_value(value.to_string())
        }
    }
}

impl Debug for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Some(ref s) => write!(f, "{s}"),
            None => write!(f, ""),
        }
    }
}

impl redb::Key for Id {
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        data1.cmp(data2)
    }
}

impl redb::Value for Id {
    type AsBytes<'a> = 'a [u8];
    type SelfType<'a> = &'a Self;
        where
            Self: 'a;
    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'b,
    {
        value.0.as_bytes()
    }
}

impl Id {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_value(value: impl Into<String>) -> Self {
        Self(Some(value.into()))
    }

    pub fn generate() -> Self {
        Self::with_value(slug())
    }

    pub const fn is_some(&self) -> bool {
        self.0.is_some()
    }

    pub const fn is_none(&self) -> bool {
        self.0.is_none()
    }
}
