//! # minidb
//!
//! Minimalistic file-based database written in Rust
//!
//! ## Features
//!
//! * File-based
//! * Thread-safe due to [`parking_lot's`](parking_lot) [`RwLock`]

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

#![forbid(unsafe_code)]
#![warn(clippy::pedantic, missing_docs, missing_debug_implementations)]

use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Result;
use parking_lot::RwLock;

/// Database client
#[derive(Debug, Clone)]
pub struct Database {
    path: Arc<RwLock<PathBuf>>,
    file_lock: Arc<RwLock<()>>,
}

impl Database {
    /// Creates a new database builder
    #[must_use]
    pub fn builder() -> DatabaseBuilder {
        DatabaseBuilder::default()
    }

    // ----------------------
    // END OF BUILDER METHODS
    // ----------------------
}

/// A builder for [Database]
#[derive(Debug, Default)]
pub struct DatabaseBuilder {
    path: Option<PathBuf>,
    tables: HashSet<String>,
}

impl DatabaseBuilder {
    /// Creates a new database builder
    ///
    /// ## Arguments
    ///
    /// * `path` - The path to the database
    ///
    /// ## Returns
    ///
    /// A new [`DatabaseBuilder`]
    pub fn new<P>(path: P) -> Self
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();

        Self {
            path: Some(path.to_path_buf()),
            tables: HashSet::new(),
        }
    }

    // ------------------------
    // START OF BUILDER METHODS
    // ------------------------

    /// Sets the database path
    ///
    /// ## Arguments
    ///
    /// * `path` - The path to the database
    #[must_use]
    pub fn path<P>(mut self, path: P) -> Self
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();

        self.path = Some(path.to_path_buf());
        self
    }

    /// Adds a table to the database
    #[must_use]
    pub fn table<T>(mut self) -> Self {
        todo!()
    }

    // ----------------------
    // END OF BUILDER METHODS
    // ----------------------

    /// Builds the database
    ///
    /// ## Returns
    ///
    /// A database client
    pub fn build(self) -> Result<Database> {
        todo!()
    }
}
