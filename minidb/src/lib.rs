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

mod errors;
mod traits;

use std::{
    collections::HashSet,
    fs::create_dir_all,
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

#[cfg(test)]
mod tests {
    use super::*;
    use minidb_utils::read_from_file;
    use tempfile::tempdir;

    #[derive(Debug, Table, Serialize, Deserialize, PartialEq)]
    struct Person {
        #[key]
        id: Id<Self>,
        name: String,
        age: u8,
    }

    #[test]
    fn test_database_new() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let temp_path = temp_dir.path();
        let db = Database::builder()
            .path(temp_path)
            .table::<Person>()
            .build()
            .expect("Failed to build database");

        assert!(dbg!(db).path().is_dir());
        assert!(temp_path.join(Person::name()).is_dir());
    }

    #[test]
    fn test_database_add_record() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let temp_path = temp_dir.path();
        let db = Database::builder()
            .path(temp_path)
            .table::<Person>()
            .build()
            .expect("Failed to build database");

        let mut p = Person {
            id: Id::default(),
            name: String::from("John Doe"),
            age: 31,
        };
        let id = p.insert(&dbg!(db)).expect("Failed to insert person");
        p.id = id;

        let str =
            read_from_file(temp_path.join("person").join(&p.id)).expect("Failed to read file");
        assert_eq!(str, format!("\0\u{8}John Doe\u{1f}"));
    }

    #[test]
    fn test_database_get_record() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let temp_path = temp_dir.path();
        let db = Database::builder()
            .path(temp_path)
            .table::<Person>()
            .build()
            .expect("Failed to create database");

        let mut p = Person {
            id: Id::new(),
            name: String::from("John Doe"),
            age: 31,
        };

        let id = p.insert(dbg!(&db)).expect("Failed to insert person");
        p.id = id;

        let p2 = Person::get(&db, dbg!(&p.id)).expect("Failed to get person");
        assert_eq!(p2, p);
    }

    #[test]
    fn test_database_update_record() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let temp_path = temp_dir.path();
        let db = Database::builder()
            .path(temp_path)
            .table::<Person>()
            .build()
            .expect("Failed to create database");

        let mut p = Person {
            id: Id::new(),
            name: String::from("John Doe"),
            age: 31,
        };

        let id = p.insert(dbg!(&db)).expect("Failed to insert person");
        p.id = id;

        p.age += 1;
        p.update(&db).expect("Failed to update person");

        let p2 = Person::get(&db, &p.id).expect("Failed to get person");
        assert_eq!(p2, p);
    }

    #[test]
    fn test_database_delete_record() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let temp_path = temp_dir.path();
        let db = Database::builder()
            .path(temp_path)
            .table::<Person>()
            .build()
            .expect("Failed to create database");

        let p = Person {
            id: Id::new(),
            name: String::from("John Doe"),
            age: 31,
        };

        let id = p.insert(dbg!(&db)).expect("Failed to insert person");

        Person::delete(&db, dbg!(&id)).expect("Failed to delete person");

        assert!(Person::get(&db, &id).is_err());
    }

    #[test]
    fn test_database_macros() {
        #![allow(dead_code)]

        struct NotTable1;

        #[derive(Table, Serialize, Deserialize)]
        struct NotTable2 {
            #[key]
            id: Id<Self>,
        }

        #[derive(Table, Serialize, Deserialize)]
        #[minidb(name = "people")]
        struct PersonTest {
            #[key]
            id: Id<Self>,

            name: String,

            age: Age,

            #[serde(skip)]
            other_name: String,
        }

        #[derive(Serialize, Deserialize)]
        struct Age(u8);

        assert_eq!(NotTable2::name(), "not_table2");
        assert_eq!(PersonTest::name(), "people");
    }

    #[test]
    fn test_database_relationship() {
        #[derive(Debug, Table, Serialize, Deserialize)]
        struct Order {
            #[key]
            id: Id<Self>,

            #[foreign_key]
            customer_id: Id<Person>,
        }

        let temp_dir = tempdir().expect("Failed to create temp dir");
        let temp_path = temp_dir.path();
        let db = Database::builder()
            .path(temp_path)
            .table::<Person>()
            .table::<Order>()
            .build()
            .expect("Failed to build database");

        dbg!(&db);

        let mut p = Person {
            id: Id::new(),
            name: "John Doe".into(),
            age: 31,
        };

        assert_eq!(Order::get_foreign_keys().len(), 1);

        p.id = p.insert(&db).expect("Failed to insert person");
        dbg!(&p);

        let mut o = Order {
            id: Id::new(),
            customer_id: p.id.clone(),
        };

        o.id = o.insert(&db).expect("Failed to insert order");
        dbg!(&o);

        assert_eq!(o.customer_id, p.id);

        p.age = 32;
        p.update(&db).expect("Failed to update person");

        let p2 = Person::get(&db, &p.id).expect("Failed to get person");
        assert_eq!(p2, p);

        Order::delete(&db, &o.id).expect("Failed to delete order");
        Person::delete(&db, &p.id).expect("Failed to delete person");
    }
}
