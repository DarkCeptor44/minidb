//! # minidb
//!
//! Minimalistic file-based database written in Rust
//!
//! ## Features
//!
//! * File-based
//! * Thread-safe due to [`parking_lot's`](parking_lot) [`RwLock`]
//!
//! ## Benchmarks
//!
//! ### Database
//!
//! ```text
//! Timer precision: 100 ns
//! database        fastest       │ slowest       │ median        │ mean          │ samples │ iters
//! ├─ delete                     │               │               │               │         │
//! │  ├─ t=1       202 µs        │ 579.3 µs      │ 248.9 µs      │ 272.1 µs      │ 100     │ 100
//! │  ├─ t=4       393.4 µs      │ 1.578 ms      │ 781.3 µs      │ 795.1 µs      │ 100     │ 100
//! │  ├─ t=8       726.6 µs      │ 4.033 ms      │ 1.559 ms      │ 1.721 ms      │ 104     │ 104
//! │  ╰─ t=16      1.502 ms      │ 5.455 ms      │ 2.978 ms      │ 3.035 ms      │ 112     │ 112
//! ├─ get                        │               │               │               │         │
//! │  ├─ t=1       135.1 µs      │ 805.3 µs      │ 169.1 µs      │ 185.8 µs      │ 100     │ 100
//! │  ├─ t=4       390.5 µs      │ 954.4 µs      │ 547.1 µs      │ 571.8 µs      │ 100     │ 100
//! │  ├─ t=8       515.4 µs      │ 3.133 ms      │ 905.2 µs      │ 1.095 ms      │ 104     │ 104
//! │  ╰─ t=16      592.2 µs      │ 2.76 ms       │ 2.055 ms      │ 2.063 ms      │ 112     │ 112
//! ├─ insert_1000                │               │               │               │         │
//! │  ├─ t=1       800.2 ms      │ 993.8 ms      │ 900.7 ms      │ 897.2 ms      │ 100     │ 100
//! │  ├─ t=4       3.437 s       │ 4.123 s       │ 3.797 s       │ 3.804 s       │ 100     │ 100
//! │  ├─ t=8       7.047 s       │ 8.154 s       │ 7.507 s       │ 7.519 s       │ 104     │ 104
//! │  ╰─ t=16      13.85 s       │ 15.7 s        │ 14.77 s       │ 14.74 s       │ 112     │ 112
//! ├─ insert_one                 │               │               │               │         │
//! │  ├─ t=1       617.4 µs      │ 2.26 ms       │ 832.7 µs      │ 867.5 µs      │ 100     │ 100
//! │  ├─ t=4       1.067 ms      │ 8.643 ms      │ 2.603 ms      │ 2.8 ms        │ 100     │ 100
//! │  ├─ t=8       1.799 ms      │ 8.792 ms      │ 4.47 ms       │ 4.514 ms      │ 104     │ 104
//! │  ╰─ t=16      3.708 ms      │ 15.63 ms      │ 9.274 ms      │ 9.204 ms      │ 112     │ 112
//! ├─ new                        │               │               │               │         │
//! │  ├─ t=1       1.198 ms      │ 2.589 ms      │ 1.631 ms      │ 1.638 ms      │ 100     │ 100
//! │  ├─ t=4       2.639 ms      │ 4.124 ms      │ 3.289 ms      │ 3.306 ms      │ 100     │ 100
//! │  ├─ t=8       4.06 ms       │ 6.823 ms      │ 5.713 ms      │ 5.709 ms      │ 104     │ 104
//! │  ╰─ t=16      6.969 ms      │ 22.62 ms      │ 10.51 ms      │ 11.71 ms      │ 112     │ 112
//! ╰─ update                     │               │               │               │         │
//!    ├─ t=1       720.6 µs      │ 2.704 ms      │ 1.003 ms      │ 1.043 ms      │ 100     │ 100
//!    ├─ t=4       1.283 ms      │ 20.63 ms      │ 2.659 ms      │ 2.859 ms      │ 100     │ 100
//!    ├─ t=8       1.568 ms      │ 9.063 ms      │ 4.801 ms      │ 4.883 ms      │ 104     │ 104
//!    ╰─ t=16      3.184 ms      │ 21.92 ms      │ 10.22 ms      │ 10.52 ms      │ 112     │ 112
//! ```

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

mod errors;
mod traits;

use std::{
    collections::HashSet,
    fs::{create_dir_all, remove_file},
    path::{Path, PathBuf},
    sync::Arc,
};

pub use errors::DBError;
pub use minidb_macros::Table;
pub use traits::{AsTable, Id};

use anyhow::{Context, Result};
use minidb_utils::{PathExt, deserialize_file, serialize_file};
use parking_lot::{RwLock, RwLockReadGuard};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};

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

    /// Deletes a record from a table
    ///
    /// ## Arguments
    ///
    /// * `id` - ID of the record to delete
    ///
    /// ## Errors
    ///
    /// * [`DBError::InvalidKey`]: Invalid key
    /// * [`DBError::FailedToReadMetadata`]: Failed to read metadata
    /// * [`DBError::NoMetadata`]: Metadata not found
    /// * [`DBError::NoTables`]: No tables were found in the database
    /// * [`DBError::RecordNotFound`]: Record not found
    /// * [`DBError::FailedToRemoveFile`]: Failed to remove file
    pub fn delete<T>(&self, id: &Id<T>) -> Result<()>
    where
        T: AsTable,
    {
        if id.is_none() {
            return Err(DBError::InvalidKey(id.to_string()).into());
        }

        let meta = self
            .metadata()
            .context(DBError::FailedToReadMetadata)?
            .context(DBError::NoMetadata)?;

        if meta.tables.is_empty() {
            return Err(DBError::NoTables.into());
        }

        // TODO restrict deleting record if other tables have foreign keys pointing to it

        let table_name = T::name();
        let path = self.path.read();
        let file_path = path.join(table_name).join(id);

        if !file_path.is_file() {
            return Err(DBError::RecordNotFound {
                table: table_name.to_string(),
                id: id.to_string(),
            }
            .into());
        }

        let _lock = self.file_lock.write();
        remove_file(&file_path).context(DBError::FailedToRemoveFile(file_path))?;

        Ok(())
    }

    /// Gets a record from a table
    ///
    /// ## Arguments
    ///
    /// * `id` - ID of the record to get
    ///
    /// ## Returns
    ///
    /// A record of type `T` where `T` implements [`AsTable`]
    ///
    /// ## Errors
    ///
    /// * [`DBError::InvalidKey`]: Invalid key
    /// * [`DBError::FailedToReadMetadata`]: Failed to read metadata
    /// * [`DBError::NoMetadata`]: Metadata not found
    /// * [`DBError::NoTables`]: No tables were found in the database
    /// * [`DBError::RecordNotFound`]: Record not found
    /// * [`DBError::FailedToDeserializeFile`]: Failed to deserialize file
    pub fn get<T>(&self, id: &Id<T>) -> Result<T>
    where
        T: AsTable + for<'de> Deserialize<'de>,
    {
        if id.is_none() {
            return Err(DBError::InvalidKey(id.to_string()).into());
        }

        let meta = self
            .metadata()
            .context(DBError::FailedToReadMetadata)?
            .context(DBError::NoMetadata)?;

        if meta.tables.is_empty() {
            return Err(DBError::NoTables.into());
        }

        let table_name = T::name();
        let path = self.path.read();
        let table_dir_path = path.join(table_name);
        let file_path = table_dir_path.join(id);

        if !file_path.is_file() {
            return Err(DBError::RecordNotFound {
                table: table_name.to_string(),
                id: id.to_string(),
            }
            .into());
        }

        let _lock = self.file_lock.read();
        let mut record: T =
            deserialize_file(&file_path).context(DBError::FailedToDeserializeFile(file_path))?;

        record.set_id(id.clone());

        Ok(record)
    }

    /// Inserts a record into the table and returns the ID
    ///
    /// ID will be generated automatically
    ///
    /// ## Arguments
    ///
    /// * `record` - The record to insert
    ///
    /// ## Errors
    ///
    /// * [`DBError::FailedToReadMetadata`]: Failed to read metadata
    /// * [`DBError::NoMetadata`]: Metadata not found
    /// * [`DBError::NoTables`]: No tables were found in the database
    /// * [`DBError::RecordAlreadyExists`]: Record already exists
    /// * [`DBError::ForeignKeyViolation`]: Referenced record does not exist
    /// * [`DBError::InvalidForeignKey`]: Referenced record does not exist
    /// * [`DBError::FailedToCreateTableDir`]: Failed to create table directory
    /// * [`DBError::FailedToSerializeFile`]: Failed to serialize file
    pub fn insert<T>(&self, record: &T) -> Result<Id<T>>
    where
        T: AsTable + Serialize,
    {
        let meta = self
            .metadata()
            .context(DBError::FailedToReadMetadata)?
            .context(DBError::NoMetadata)?;

        if meta.tables.is_empty() {
            return Err(DBError::NoTables.into());
        }

        let table_name = T::name();
        if let Some(id) = &record.get_id().value {
            return Err(DBError::RecordAlreadyExists {
                table: table_name.to_string(),
                id: id.clone(),
            }
            .into());
        }

        for (field_name, ref_table, get_fk_id) in T::get_foreign_keys() {
            let fk_id_option = get_fk_id(record);
            if let Some(fk_id_str) = fk_id_option {
                if !self.record_exists(ref_table, fk_id_str) {
                    return Err(DBError::ForeignKeyViolation {
                        field: field_name.to_string(),
                        table: ref_table.to_string(),
                        id: fk_id_option.unwrap_or("").to_string(),
                    }
                    .into());
                }
            } else {
                return Err(DBError::InvalidForeignKey {
                    field: field_name.to_string(),
                    table: ref_table.to_string(),
                    id: fk_id_option.unwrap_or("").to_string(),
                }
                .into());
            }
        }

        let path = self.path.read();
        let table_dir_path = path.join(table_name);

        create_dir_all(&table_dir_path)
            .context(DBError::FailedToCreateTableDir(table_dir_path.clone()))?;

        let id = Id::generate();
        let file_path = table_dir_path.join(&id);

        if file_path.is_file() {
            return Err(DBError::RecordAlreadyExists {
                table: table_name.to_string(),
                id: id.to_string(),
            }
            .into());
        }

        let _lock = self.file_lock.write();
        serialize_file(&file_path, record).context(DBError::FailedToSerializeFile(file_path))?;

        Ok(id)
    }

    /// Returns the metadata of the database
    fn metadata(&self) -> Result<Option<Metadata>> {
        let path_guard = self.path.read();
        let path = path_guard.as_path();
        let file_path = path.join("metadata");

        if !file_path.is_file() {
            return Ok(None);
        }

        let _lock = self.file_lock.read();
        let data: Metadata = deserialize_file(file_path).context(DBError::FailedToReadMetadata)?;

        Ok(Some(data))
    }

    /// Returns a read guard to the database path.
    ///
    /// The guard dereferences to a [`&PathBuf`](PathBuf) or [`&Path`](Path)
    pub fn path(&self) -> RwLockReadGuard<'_, PathBuf> {
        self.path.read()
    }

    /// Checks if a record exists in the database
    #[must_use]
    pub fn record_exists(&self, table_name: &str, id_str: &str) -> bool {
        let path = self.path.read();
        let file_path = path.join(table_name).join(id_str);
        file_path.is_file()
    }

    /// Updates a record in the table
    ///
    /// ## Arguments
    ///
    /// * `db` - The database instance
    ///
    /// ## Errors
    ///
    /// * [`DBError::InvalidKey`]: Invalid key
    /// * [`DBError::FailedToReadMetadata`]: Failed to read metadata
    /// * [`DBError::NoMetadata`]: Metadata not found
    /// * [`DBError::NoTables`]: No tables were found in the database
    /// * [`DBError::ForeignKeyViolation`]: Referenced record does not exist
    /// * [`DBError::InvalidForeignKey`]: Referenced record does not exist
    /// * [`DBError::FailedToCreateTableDir`]: Failed to create table directory
    /// * [`DBError::RecordNotFound`]: Record not found
    /// * [`DBError::FailedToSerializeFile`]: Failed to serialize file
    pub fn update<T>(&self, record: &T) -> Result<()>
    where
        T: AsTable + Serialize,
    {
        let id = record.get_id();

        if id.is_none() {
            return Err(DBError::InvalidKey(id.to_string()).into());
        }

        let meta = self
            .metadata()
            .context(DBError::FailedToReadMetadata)?
            .context(DBError::NoMetadata)?;

        if meta.tables.is_empty() {
            return Err(DBError::NoTables.into());
        }

        for (field_name, ref_table, get_fk_id) in T::get_foreign_keys() {
            let fk_id_option = get_fk_id(record);
            if let Some(fk_id_str) = fk_id_option {
                if !self.record_exists(ref_table, fk_id_str) {
                    return Err(DBError::ForeignKeyViolation {
                        field: field_name.to_string(),
                        table: ref_table.to_string(),
                        id: fk_id_option.unwrap_or("").to_string(),
                    }
                    .into());
                }
            } else {
                return Err(DBError::InvalidForeignKey {
                    field: field_name.to_string(),
                    table: ref_table.to_string(),
                    id: fk_id_option.unwrap_or("").to_string(),
                }
                .into());
            }
        }

        let table_name = T::name();
        let path = self.path.read();
        let table_dir_path = path.join(table_name);

        create_dir_all(&table_dir_path)
            .context(DBError::FailedToCreateTableDir(table_dir_path.clone()))?;

        let file_path = table_dir_path.join(id);
        if !file_path.is_file() {
            return Err(DBError::RecordNotFound {
                table: table_name.to_string(),
                id: id.to_string(),
            }
            .into());
        }

        let _lock = self.file_lock.write();
        serialize_file(&file_path, record).context(DBError::FailedToSerializeFile(file_path))
    }

    /// Writes the metadata of the database
    fn write_metadata(&self, meta: &Metadata) -> Result<()> {
        let path_guard = self.path.read();
        let path = path_guard.as_path();
        let file_path = path.join("metadata");

        let _lock = self.file_lock.write();
        serialize_file(file_path, meta).context(DBError::FailedToSerializeMetadata)?;

        Ok(())
    }
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

    /// Adds a table to the database.
    ///
    /// The table must implement the [`AsTable`] trait
    #[must_use]
    pub fn table<T>(mut self) -> Self
    where
        T: AsTable,
    {
        let table_name = T::name();

        self.tables.insert(table_name.to_string());
        self
    }

    // ----------------------
    // END OF BUILDER METHODS
    // ----------------------

    /// Builds the database
    ///
    /// ## Returns
    ///
    /// A database client
    ///
    /// ## Errors
    ///
    /// * [`DBError::NoDatabasePath`]: No path was provided for the database
    /// * [`DBError::FolderExists`]: Folder already exists and is not empty
    /// * [`DBError::NoTables`]: No tables were provided
    /// * [`DBError::FailedToCreateDatabase`]: Failed to create database directory
    /// * [`DBError::FailedToReadMetadata`]: Failed to read metadata
    /// * [`DBError::FailedToWriteMetadata`]: Failed to write metadata
    /// * [`DBError::FailedToCreateTableDir`]: Failed to create table directory
    pub fn build(self) -> Result<Database> {
        let path = self.path.ok_or(DBError::NoDatabasePath)?;

        if !path.is_empty()? {
            return Err(DBError::FolderExists(path.clone()).into());
        }

        if self.tables.is_empty() {
            return Err(DBError::NoTables.into());
        }

        create_dir_all(&path).context(DBError::FailedToCreateDatabase(path.clone()))?;

        let db = Database {
            path: Arc::new(RwLock::new(path.clone())),
            file_lock: Arc::new(RwLock::new(())),
        };
        let meta =
            if let Some(meta) = Database::metadata(&db).context(DBError::FailedToReadMetadata)? {
                meta
            } else {
                let m = Metadata {
                    tables: self.tables,
                };

                db.write_metadata(&m)
                    .context(DBError::FailedToWriteMetadata)?;
                m
            };

        meta.tables
            .par_iter()
            .map(|table| {
                let table_path = path.join(table);
                create_dir_all(&table_path).context(DBError::FailedToCreateTableDir(table_path))?;
                Ok(())
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(db)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Metadata {
    tables: HashSet<String>,
}
