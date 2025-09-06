// Copyright (c) 2025, DarkCeptor44
//
// This file is licensed under the GNU Lesser General Public License
// (either version 3 or, at your option, any later version).
//
// This software comes without any warranty, express or implied. See the
// GNU Lesser General Public License for details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with this software. If not, see <https://www.gnu.org/licenses/>.

//! # Traits
//!
//! Some traits for minidb

use std::{
    fmt::{Debug, Display},
    path::Path,
};

use cuid2::slug;
use serde::{Deserialize, Serialize};

type ForeignKeyTuple<S> = (
    &'static str,
    &'static str,
    Box<dyn Fn(&S) -> Option<&str> + Send + Sync>,
);

/// A trait for defining a table
pub trait AsTable: Sized {
    /// The name of the table in `snake_case`
    fn name() -> &'static str;

    /// The primary key of the table
    fn get_id(&self) -> &Id<Self>;

    /// Sets the primary key of the table
    fn set_id(&mut self, id: Id<Self>);

    /// The foreign keys of the table
    fn get_foreign_keys() -> Vec<ForeignKeyTuple<Self>>;
}

/// Represents the ID of a record
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Id<T> {
    /// The underlying value
    pub value: Option<String>,

    _phantom: std::marker::PhantomData<T>,
}

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<S, T> From<S> for Id<T>
where
    S: AsRef<str>,
{
    fn from(value: S) -> Self {
        let value = value.as_ref().trim();
        if value.is_empty() {
            Id::default()
        } else {
            Id::with_value(Some(value.to_string()))
        }
    }
}

impl<T> AsRef<Path> for Id<T> {
    fn as_ref(&self) -> &Path {
        match self.value {
            Some(ref s) => Path::new(s),
            None => Path::new(""),
        }
    }
}

impl<T> Default for Id<T> {
    fn default() -> Self {
        Self {
            value: None,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T> Debug for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.value)
    }
}

impl<T> Display for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.value {
            Some(ref s) => write!(f, "{s}"),
            None => write!(f, ""),
        }
    }
}

impl<T> Id<T> {
    /// Creates a new ID with [None]
    #[must_use]
    pub fn new() -> Self {
        Self::with_value(None)
    }

    /// Creates a new ID with a [Option]
    #[must_use]
    pub fn with_value(value: Option<String>) -> Self {
        Self {
            value,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Generates a new ID
    #[must_use]
    pub fn generate() -> Self {
        Self::with_value(Some(slug()))
    }

    /// Returns `true` if the ID is [`Some`]
    #[must_use]
    pub const fn is_some(&self) -> bool {
        self.value.is_some()
    }

    /// Returns `true` if the ID is [`None`]
    #[must_use]
    pub const fn is_none(&self) -> bool {
        self.value.is_none()
    }
}
