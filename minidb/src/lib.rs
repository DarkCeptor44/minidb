//! # minidb
//!
//! Minimalistic file-based database written in Rust
//!
//! ## Features
//!
//! * File-based, this means the tables are sub-directories and the records are files
//! * Uses [bitcode](https://crates.io/crates/bitcode) as the binary format to store the data
//! * Uses [cuid2] slugs for record IDs
//! * Easy table definition with procedural macros
//! * Built around poison-free read-write locks to be thread-safe
//! * Relies on [serde] for serialization and deserialization of the tables
//!
//! ## Why not async
//!
//! The database was initially built without async, then I thought about it and wrote async versions of each filesystem-related function in [minidb-utils](minidb_utils) but ultimately decided not to do it because there's no proper benchmark for concurrent async yet, I'd assume the overhead from async wouldn't be worth it and the API would be more complex, for example adding a table to the database instance could go from:
//!
//! ```rust,ignore
//! let db = Database::builder().path(path).table::<Person>().build().unwrap();
//! ```
//!
//! To:
//!
//! ```rust,ignore
//! let db = Database::builder().path(path).await.table::<Person>().await.build().await.unwrap();
//! ```
//!
//! However, it's not impossible if future benchmarks show enough difference.
//!
//! ## MSRV
//!
//! The minimum supported Rust version is `1.85.0`. The MSRV might be changed at any time with a minor version bump
//!
//! ## Installation
//!
//! Add the following to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! minidb = "^0.1"
//! serde = { version = "^1", features = ["derive"] }
//! ```
//!
//! ## Usage
//!
//! A minimal example of how to use minidb is provided in `examples/simple.rs`, you can run it with:
//!
//! ```bash
//! cargo run -p minidb --example simple
//!
//! # or
//! cd minidb
//! cargo run --example simple
//! ```
//!
//! The example code:
//!
//! ```rust,ignore
//! use minidb::{AsTable, Database, Id, Table};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, Table, Serialize, Deserialize, PartialEq)]
//! struct Person {
//!     #[key]
//!     id: Id<Self>,
//!     name: String,
//!     age: u8,
//! }
//!
//! // 1. Create database
//! let db = Database::builder()
//!     .path("path/to/db")
//!     .table::<Person>()
//!     .build()
//!     .unwrap();
//!
//! // 2. Insert a new person
//! let mut person_to_insert = Person {
//!     id: Id::new(),
//!     name: "John Doe".to_string(),
//!     age: 31,
//! };
//! let id = db.insert(&person_to_insert).unwrap();
//! person_to_insert.id = id;
//! println!("Inserted person: {:?}", person_to_insert);
//!
//! // 3. Retrieve person
//! let person_retrieved = db.get(&person_to_insert.id).unwrap();
//! assert_eq!(person_retrieved, person_to_insert);
//! println!(
//!     "Successfully retrieved and verified person: {:?}",
//!     person_retrieved
//! );
//!
//! // 4. Update person's age
//! person_to_insert.age += 1;
//! db.update(&person_to_insert).unwrap();
//! println!("Updated person: {:?}", person_to_insert);
//!
//! // 5. Retrieve updated person
//! let person_retrieved = db.get(&person_to_insert.id).unwrap();
//! assert_eq!(person_retrieved.age, 32);
//! println!(
//!     "Successfully retrieved and verified updated person: {:?}",
//!     person_retrieved
//! );
//!
//! // 6. Delete person
//! db.delete(&person_to_insert.id).unwrap();
//! println!("Deleted person");
//!
//! // 7. Verify person is deleted
//! let user_deleted = db.get(&person_to_insert.id);
//! assert!(user_deleted.is_err());
//! println!("Verified deletion");
//!
//! println!("\nExample completed successfully");
//! ```
//!
//! ## Audits
//!
//! * From [cargo-audit](https://crates.io/crates/cargo-audit):
//!
//! ```text
//! Crate:     atomic-polyfill
//! Version:   1.0.3
//! Warning:   unmaintained
//! Title:     atomic-polyfill is unmaintained
//! Date:      2023-07-11
//! ID:        RUSTSEC-2023-0089
//! URL:       https://rustsec.org/advisories/RUSTSEC-2023-0089
//! Dependency tree:
//! atomic-polyfill 1.0.3
//! └── heapless 0.7.17
//!     └── postcard 1.1.3
//!         └── minidb-utils 0.1.0
//!             └── minidb 0.1.0
//!
//! Crate:     paste
//! Version:   1.0.15
//! Warning:   unmaintained
//! Title:     paste - no longer maintained
//! Date:      2024-10-07
//! ID:        RUSTSEC-2024-0436
//! URL:       https://rustsec.org/advisories/RUSTSEC-2024-0436
//! Dependency tree:
//! paste 1.0.15
//! ├── rmp 0.8.14
//! │   └── rmp-serde 1.3.0
//! │       └── minidb-utils 0.1.0
//! │           └── minidb 0.1.0
//! └── minidb 0.1.0
//!
//! Crate:     serde_cbor
//! Version:   0.11.2
//! Warning:   unmaintained
//! Title:     serde_cbor is unmaintained
//! Date:      2021-08-15
//! ID:        RUSTSEC-2021-0127
//! URL:       https://rustsec.org/advisories/RUSTSEC-2021-0127
//! Dependency tree:
//! serde_cbor 0.11.2
//! └── minidb-utils 0.1.0
//!     └── minidb 0.1.0
//!
//! warning: 3 allowed warnings found
//! ```
//!
//! ## Tests
//!
//! The tests can be run with:
//!
//! ```bash
//! cargo test -p minidb
//! ```
//!
//! ## Benchmarks
//!
//! ### Database
//!
//! ```text
//! Timer precision: 100 ns
//! database    fastest       │ slowest       │ median        │ mean          │ samples │ iters
//! ├─ delete                 │               │               │               │         │
//! │  ├─ 1                   │               │               │               │         │
//! │  │  ├─ t=1  198.6 µs      │ 312.4 µs      │ 203.5 µs      │ 208.6 µs      │ 100     │ 100
//! │  │  ├─ t=4  414.7 µs      │ 924.2 µs      │ 653.5 µs      │ 644.5 µs      │ 100     │ 100
//! │  │  ├─ t=8  731 µs        │ 2.041 ms      │ 1.308 ms      │ 1.286 ms      │ 104     │ 104
//! │  │  ╰─ t=16  1.365 ms      │ 3.538 ms      │ 2.408 ms      │ 2.401 ms      │ 112     │ 112
//! │  ╰─ 1000                   │               │               │               │         │
//! │     ├─ t=1   200.2 ms      │ 244.6 ms      │ 212 ms        │ 214.1 ms      │ 100     │ 100
//! │     ├─ t=4   754.9 ms      │ 846.4 ms      │ 787.5 ms      │ 787.6 ms      │ 100     │ 100
//! │     ├─ t=8   1.57 s        │ 1.767 s       │ 1.653 s       │ 1.651 s       │ 104     │ 104
//! │     ╰─ t=16  3.413 s       │ 3.829 s       │ 3.458 s       │ 3.538 s       │ 112     │ 112
//! ├─ exists                    │               │               │               │         │
//! │  ├─ 1                      │               │               │               │         │
//! │  │  ├─ t=1   21.09 µs      │ 77.49 µs      │ 21.59 µs      │ 23.08 µs      │ 100     │ 100
//! │  │  ├─ t=4   35.89 µs      │ 160.5 µs      │ 98.79 µs      │ 102.1 µs      │ 100     │ 100
//! │  │  ├─ t=8   118.6 µs      │ 279.7 µs      │ 184.3 µs      │ 186.2 µs      │ 104     │ 104
//! │  │  ╰─ t=16  65.59 µs      │ 563.7 µs      │ 462.9 µs      │ 439.2 µs      │ 112     │ 112
//! │  ╰─ 1000                   │               │               │               │         │
//! │     ├─ t=1   21.12 ms      │ 30.87 ms      │ 21.75 ms      │ 22.2 ms       │ 100     │ 100
//! │     ├─ t=4   68.53 ms      │ 84.88 ms      │ 74.01 ms      │ 73.97 ms      │ 100     │ 100
//! │     ├─ t=8   146.1 ms      │ 185.4 ms      │ 160.6 ms      │ 164 ms        │ 104     │ 104
//! │     ╰─ t=16  327.4 ms      │ 403.2 ms      │ 366 ms        │ 362 ms        │ 112     │ 112
//! ├─ get                       │               │               │               │         │
//! │  ├─ 1                      │               │               │               │         │
//! │  │  ├─ t=1   130.3 µs      │ 545.8 µs      │ 131.8 µs      │ 137.8 µs      │ 100     │ 100
//! │  │  ├─ t=4   283.3 µs      │ 618.4 µs      │ 476.5 µs      │ 474 µs        │ 100     │ 100
//! │  │  ├─ t=8   578.7 µs      │ 1.297 ms      │ 942.7 µs      │ 939.7 µs      │ 104     │ 104
//! │  │  ╰─ t=16  864.5 µs      │ 2.236 ms      │ 1.933 ms      │ 1.845 ms      │ 112     │ 112
//! │  ╰─ 1000                   │               │               │               │         │
//! │     ├─ t=1   129.7 ms      │ 157.7 ms      │ 131.8 ms      │ 133.6 ms      │ 100     │ 100
//! │     ├─ t=4   279 ms        │ 345 ms        │ 301.8 ms      │ 305.7 ms      │ 100     │ 100
//! │     ├─ t=8   567 ms        │ 619.6 ms      │ 579.9 ms      │ 588.7 ms      │ 104     │ 104
//! │     ╰─ t=16  1.186 s       │ 1.334 s       │ 1.2 s         │ 1.222 s       │ 112     │ 112
//! ├─ insert                    │               │               │               │         │
//! │  ├─ 1                      │               │               │               │         │
//! │  │  ├─ t=1   609.7 µs      │ 5.124 ms      │ 714.3 µs      │ 787.1 µs      │ 100     │ 100
//! │  │  ├─ t=4   1.025 ms      │ 3.398 ms      │ 2.05 ms       │ 2.019 ms      │ 100     │ 100
//! │  │  ├─ t=8   1.45 ms       │ 6.377 ms      │ 3.825 ms      │ 3.718 ms      │ 104     │ 104
//! │  │  ╰─ t=16  2.644 ms      │ 12.6 ms       │ 6.949 ms      │ 7.036 ms      │ 112     │ 112
//! │  ╰─ 1000                   │               │               │               │         │
//! │     ├─ t=1   680.2 ms      │ 945.2 ms      │ 728.2 ms      │ 734.9 ms      │ 100     │ 100
//! │     ├─ t=4   2.674 s       │ 2.917 s       │ 2.766 s       │ 2.774 s       │ 100     │ 100
//! │     ├─ t=8   5.276 s       │ 8.274 s       │ 5.393 s       │ 5.704 s       │ 104     │ 104
//! │     ╰─ t=16  10.51 s       │ 11.67 s       │ 10.98 s       │ 11.1 s        │ 112     │ 112
//! ├─ new                       │               │               │               │         │
//! │  ├─ t=1      1.171 ms      │ 1.547 ms      │ 1.233 ms      │ 1.254 ms      │ 100     │ 100
//! │  ├─ t=4      2.218 ms      │ 3.641 ms      │ 2.561 ms      │ 2.585 ms      │ 100     │ 100
//! │  ├─ t=8      3.748 ms      │ 5.438 ms      │ 4.578 ms      │ 4.58 ms       │ 104     │ 104
//! │  ╰─ t=16     6.147 ms      │ 9.398 ms      │ 8.493 ms      │ 8.267 ms      │ 112     │ 112
//! ╰─ update                    │               │               │               │         │
//!    ├─ 1                      │               │               │               │         │
//!    │  ├─ t=1   677.9 µs      │ 1.784 ms      │ 812 µs        │ 855.6 µs      │ 100     │ 100
//!    │  ├─ t=4   1.159 ms      │ 3.903 ms      │ 2.4 ms        │ 2.464 ms      │ 100     │ 100
//!    │  ├─ t=8   1.603 ms      │ 6.967 ms      │ 3.887 ms      │ 3.995 ms      │ 104     │ 104
//!    │  ╰─ t=16  2.165 ms      │ 15.49 ms      │ 8.379 ms      │ 8.435 ms      │ 112     │ 112
//!    ╰─ 1000                   │               │               │               │         │
//!       ├─ t=1   711.3 ms      │ 839.4 ms      │ 730.3 ms      │ 734.1 ms      │ 100     │ 100
//!       ├─ t=4   3.024 s       │ 3.373 s       │ 3.05 s        │ 3.078 s       │ 100     │ 100
//!       ├─ t=8   6.158 s       │ 6.343 s       │ 6.251 s       │ 6.254 s       │ 104     │ 104
//!       ╰─ t=16  12.78 s       │ 12.95 s       │ 12.85 s       │ 12.87 s       │ 112     │ 112
//! ```
//!
//! ### Id
//!
//! ```text
//! Timer precision: 100 ns
//! id              fastest       │ slowest       │ median        │ mean          │ samples │ iters
//! ╰─ id_generate  2.299 µs      │ 160 µs        │ 2.399 µs      │ 3.979 µs      │ 100     │ 100
//! ```
//!
//! ## License
//!
//! This project is licensed under the [GNU Lesser General Public License v3](https://www.gnu.org/licenses/lgpl-3.0.en.html).

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
    fmt::{Debug, Display},
    fs::{create_dir_all, remove_file},
    path::{Path, PathBuf},
    sync::Arc,
};

pub use errors::DBError;
pub use minidb_macros::Table;
pub use traits::AsTable;

use anyhow::{Context, Result};
use cuid2::slug;
use minidb_utils::{PathExt, deserialize_file, serialize_file};
use parking_lot::{RwLock, RwLockReadGuard};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};

/// A database client
///
/// ## Example
///
/// ```rust,ignore
/// use minidb::{AsTable, Database, Id, Table};
///
/// #[derive(Table)]
/// struct Person {
///     #[key]
///     id: Id<Self>,
///     name: String,
///     age: u8,
/// }
///
/// let db = Database::builder()
///     .path("path/to/db")
///     .table::<Person>()
///     .build()
///     .unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct Database {
    file_lock: Arc<RwLock<()>>,
    path: Arc<RwLock<PathBuf>>,
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
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// let id = Id::from("ldakksdlakls");
    /// db.delete(&id).unwrap();
    /// ```
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
        let file_path = path.join(table_name).join(id.to_string());

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
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// let id = Id::from("alsdklaksa");
    /// let person = db.get(&id).unwrap();
    ///
    /// println!("Person: {:?}", person);
    /// ```
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
        let file_path = table_dir_path.join(id.to_string());

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
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// let mut person_to_insert = Person {
    ///     id: Id::new(),
    ///     name: "John Doe".to_string(),
    ///     age: 31,
    /// };
    /// let id = db.insert(&person_to_insert).unwrap();
    /// person_to_insert.id = id;
    ///
    /// println!("Inserted person: {:?}", person_to_insert);
    /// ```
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
                if !self.exists_impl(ref_table, fk_id_str) {
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
        let file_path = table_dir_path.join(id.to_string());

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
    ///
    /// ## Arguments
    ///
    /// * `id` - The ID of the record to check
    ///
    /// ## Returns
    ///
    /// `true` if the record exists, `false` otherwise
    #[must_use]
    pub fn exists<T>(&self, id: &Id<T>) -> bool
    where
        T: AsTable,
    {
        self.exists_impl(T::name(), &id.to_string())
    }

    /// Checks if a record exists in the database
    fn exists_impl(&self, table_name: &str, id: &str) -> bool {
        let path = self.path.read();
        let file_path = path.join(table_name).join(id);
        file_path.is_file()
    }

    /// Updates a record in the table
    ///
    /// ## Arguments
    ///
    /// * `record` - The record to update
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
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// let mut person = Person {
    ///     id: Id::from("alskdlasla"),
    ///     name: "John Doe".to_string(),
    ///     age: 31,
    /// };
    ///
    /// person.age += 1;
    /// db.update(&person).unwrap();
    ///
    /// println!("Updated person: {:?}", person);
    /// ```
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
                if !self.exists_impl(ref_table, fk_id_str) {
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

        let file_path = table_dir_path.join(id.to_string());
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

        match path.is_empty() {
            Ok(true) => (),
            Ok(false) => return Err(DBError::FolderExists(path.clone()).into()),
            Err(e) => return Err(e),
        }

        if self.tables.is_empty() {
            return Err(DBError::NoTables.into());
        }

        create_dir_all(&path).context(DBError::FailedToCreateDatabase(path.clone()))?;

        let db = Database {
            file_lock: Arc::new(RwLock::new(())),
            path: Arc::new(RwLock::new(path.clone())),
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
