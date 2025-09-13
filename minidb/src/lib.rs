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
//! * Built with interprocess file locks for thread-safety
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
//! │  │  ├─ t=1  273.5 µs      │ 327.1 µs      │ 288.9 µs      │ 288.8 µs      │ 100     │ 100
//! │  │  ├─ t=4  477.9 µs      │ 1.368 ms      │ 915.8 µs      │ 887.8 µs      │ 100     │ 100
//! │  │  ├─ t=8  696.6 µs      │ 2.804 ms      │ 1.661 ms      │ 1.653 ms      │ 104     │ 104
//! │  │  ╰─ t=16  1.062 ms      │ 4.956 ms      │ 3.002 ms      │ 2.951 ms      │ 112     │ 112
//! │  ╰─ 1000                   │               │               │               │         │
//! │     ├─ t=1   274.3 ms      │ 298.7 ms      │ 283.9 ms      │ 285 ms        │ 100     │ 100
//! │     ├─ t=4   1.034 s       │ 1.119 s       │ 1.069 s       │ 1.068 s       │ 100     │ 100
//! │     ├─ t=8   2.035 s       │ 2.632 s       │ 2.128 s       │ 2.165 s       │ 104     │ 104
//! │     ╰─ t=16  4.339 s       │ 4.542 s       │ 4.4 s         │ 4.416 s       │ 112     │ 112
//! ├─ exists                    │               │               │               │         │
//! │  ├─ 1                      │               │               │               │         │
//! │  │  ├─ t=1   91.49 µs      │ 191.8 µs      │ 92.69 µs      │ 94.87 µs      │ 100     │ 100
//! │  │  ├─ t=4   276.2 µs      │ 425.2 µs      │ 343.6 µs      │ 348 µs        │ 100     │ 100
//! │  │  ├─ t=8   479.3 µs      │ 774.1 µs      │ 613.5 µs      │ 607.9 µs      │ 104     │ 104
//! │  │  ╰─ t=16  714.5 µs      │ 1.31 ms       │ 1.038 ms      │ 1.024 ms      │ 112     │ 112
//! │  ╰─ 1000                   │               │               │               │         │
//! │     ├─ t=1   91.47 ms      │ 100.3 ms      │ 92.27 ms      │ 92.59 ms      │ 100     │ 100
//! │     ├─ t=4   188.3 ms      │ 217.5 ms      │ 195.7 ms      │ 198.2 ms      │ 100     │ 100
//! │     ├─ t=8   466.9 ms      │ 511.9 ms      │ 509 ms        │ 504.7 ms      │ 104     │ 104
//! │     ╰─ t=16  1.024 s       │ 1.056 s       │ 1.048 s       │ 1.044 s       │ 112     │ 112
//! ├─ get                       │               │               │               │         │
//! │  ├─ 1                      │               │               │               │         │
//! │  │  ├─ t=1   205.5 µs      │ 508 µs        │ 207.8 µs      │ 213.9 µs      │ 100     │ 100
//! │  │  ├─ t=4   492.5 µs      │ 837.1 µs      │ 663.9 µs      │ 660.4 µs      │ 100     │ 100
//! │  │  ├─ t=8   635.2 µs      │ 1.518 ms      │ 1.279 ms      │ 1.237 ms      │ 104     │ 104
//! │  │  ╰─ t=16  1.039 ms      │ 2.566 ms      │ 2.297 ms      │ 2.113 ms      │ 112     │ 112
//! │  ╰─ 1000                   │               │               │               │         │
//! │     ├─ t=1   204.8 ms      │ 210.1 ms      │ 206.4 ms      │ 206.6 ms      │ 100     │ 100
//! │     ├─ t=4   376.5 ms      │ 418.2 ms      │ 392.6 ms      │ 392.8 ms      │ 100     │ 100
//! │     ├─ t=8   686.4 ms      │ 732.9 ms      │ 712 ms        │ 711.8 ms      │ 104     │ 104
//! │     ╰─ t=16  1.45 s        │ 1.587 s       │ 1.46 s        │ 1.478 s       │ 112     │ 112
//! ├─ insert                    │               │               │               │         │
//! │  ├─ 1                      │               │               │               │         │
//! │  │  ├─ t=1   698.4 µs      │ 1.132 ms      │ 717.1 µs      │ 734.9 µs      │ 100     │ 100
//! │  │  ├─ t=4   919.5 µs      │ 3.753 ms      │ 2.177 ms      │ 2.099 ms      │ 100     │ 100
//! │  │  ├─ t=8   1.221 ms      │ 7.17 ms       │ 4.168 ms      │ 3.957 ms      │ 104     │ 104
//! │  │  ╰─ t=16  1.603 ms      │ 13.72 ms      │ 7.236 ms      │ 7.239 ms      │ 112     │ 112
//! │  ╰─ 1000                   │               │               │               │         │
//! │     ├─ t=1   724.4 ms      │ 1.527 s       │ 780.7 ms      │ 819.9 ms      │ 100     │ 100
//! │     ├─ t=4   2.975 s       │ 3.191 s       │ 3.091 s       │ 3.085 s       │ 100     │ 100
//! │     ├─ t=8   5.979 s       │ 6.386 s       │ 6.154 s       │ 6.152 s       │ 104     │ 104
//! │     ╰─ t=16  12.01 s       │ 12.63 s       │ 12.35 s       │ 12.33 s       │ 112     │ 112
//! ├─ new                       │               │               │               │         │
//! │  ├─ t=1      1.485 ms      │ 1.787 ms      │ 1.534 ms      │ 1.54 ms       │ 100     │ 100
//! │  ├─ t=4      2.539 ms      │ 3.575 ms      │ 2.995 ms      │ 2.987 ms      │ 100     │ 100
//! │  ├─ t=8      4.216 ms      │ 5.654 ms      │ 5.088 ms      │ 4.999 ms      │ 104     │ 104
//! │  ╰─ t=16     7.947 ms      │ 10.85 ms      │ 9.929 ms      │ 9.728 ms      │ 112     │ 112
//! ╰─ update                    │               │               │               │         │
//!    ├─ 1                      │               │               │               │         │
//!    │  ├─ t=1   746 µs        │ 6.473 ms      │ 800 µs        │ 886.6 µs      │ 100     │ 100
//!    │  ├─ t=4   1.042 ms      │ 12.66 ms      │ 2.802 ms      │ 2.858 ms      │ 100     │ 100
//!    │  ├─ t=8   1.165 ms      │ 7.75 ms       │ 4.052 ms      │ 4.146 ms      │ 104     │ 104
//!    │  ╰─ t=16  1.586 ms      │ 15.22 ms      │ 8.026 ms      │ 8.066 ms      │ 112     │ 112
//!    ╰─ 1000                   │               │               │               │         │
//!       ├─ t=1   771 ms        │ 839.2 ms      │ 796.9 ms      │ 799 ms        │ 100     │ 100
//!       ├─ t=4   3.232 s       │ 3.406 s       │ 3.282 s       │ 3.293 s       │ 100     │ 100
//!       ├─ t=8   6.704 s       │ 6.943 s       │ 6.748 s       │ 6.792 s       │ 104     │ 104
//!       ╰─ t=16  13.91 s       │ 14.37 s       │ 14.08 s       │ 14.09 s       │ 112     │ 112
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
    fs::{File, create_dir_all, remove_file},
    path::{Path, PathBuf},
    sync::Arc,
};

pub use errors::DBError;
pub use minidb_macros::Table;
pub use traits::AsTable;

use anyhow::{Context, Result};
use cuid2::slug;
use minidb_utils::{
    ArgonParams, PathExt, derive_key, deserialize_file, generate_salt, serialize_file,
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};

/// A type alias for a 16-byte array
type Salt = [u8; 16];

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
#[derive(Debug)]
pub struct Database {
    derived_key: Arc<Option<Vec<u8>>>,
    lock_file_path: Arc<PathBuf>,
    path: Arc<PathBuf>,
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Self {
            derived_key: Arc::clone(&self.derived_key),
            lock_file_path: Arc::clone(&self.lock_file_path),
            path: Arc::clone(&self.path),
        }
    }
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
    /// * [`DBError::FailedToOpenLockFile`]: Failed to open lock file
    /// * [`DBError::FailedToLockFile`]: Failed to lock file
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
        let lock_file = self.get_lock()?;
        lock_file
            .lock()
            .context(DBError::FailedToLockFile(self.lock_file_path.to_path_buf()))?;

        if id.is_none() {
            return Err(DBError::InvalidKey(id.to_string()).into());
        }

        let meta = self
            .metadata_unlocked()
            .context(DBError::FailedToReadMetadata)?
            .context(DBError::NoMetadata)?;

        if meta.tables.is_empty() {
            return Err(DBError::NoTables.into());
        }

        // TODO restrict deleting record if other tables have foreign keys pointing to it

        let table_name = T::name();
        let path = self.path.as_path();
        let file_path = path.join(table_name).join(id.to_string());

        if !file_path.is_file() {
            return Err(DBError::RecordNotFound {
                table: table_name.to_string(),
                id: id.to_string(),
            }
            .into());
        }

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
    /// * [`DBError::FailedToOpenLockFile`]: Failed to open lock file
    /// * [`DBError::FailedToLockFile`]: Failed to lock file
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
        let lock_file = self.get_lock()?;
        lock_file
            .lock_shared()
            .context(DBError::FailedToLockFile(self.lock_file_path.to_path_buf()))?;

        if id.is_none() {
            return Err(DBError::InvalidKey(id.to_string()).into());
        }

        let meta = self
            .metadata_unlocked()
            .context(DBError::FailedToReadMetadata)?
            .context(DBError::NoMetadata)?;

        if meta.tables.is_empty() {
            return Err(DBError::NoTables.into());
        }

        let table_name = T::name();
        let path = self.path.as_path();
        let table_dir_path = path.join(table_name);
        let file_path = table_dir_path.join(id.to_string());

        if !file_path.is_file() {
            return Err(DBError::RecordNotFound {
                table: table_name.to_string(),
                id: id.to_string(),
            }
            .into());
        }

        let mut record: T =
            deserialize_file(&file_path).context(DBError::FailedToDeserializeFile(file_path))?;
        record.set_id(id.clone());

        Ok(record)
    }

    /// Gets the lock file
    fn get_lock(&self) -> Result<File> {
        // TODO use per-table locking
        File::options()
            .create(true)
            .write(true)
            .truncate(false)
            .open(self.lock_file_path.as_path())
            .context(DBError::FailedToOpenLockFile(
                self.lock_file_path.to_path_buf(),
            ))
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
    /// * [`DBError::FailedToOpenLockFile`]: Failed to open lock file
    /// * [`DBError::FailedToLockFile`]: Failed to lock file
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
        let lock_file = self.get_lock()?;
        lock_file
            .lock()
            .context(DBError::FailedToLockFile(self.lock_file_path.to_path_buf()))?;

        let meta = self
            .metadata_unlocked()
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
                if !self.exists_impl_unlocked(ref_table, fk_id_str) {
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

        let path = self.path.as_path();
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

        serialize_file(&file_path, record).context(DBError::FailedToSerializeFile(file_path))?;
        Ok(id)
    }

    /// Returns the metadata of the database
    fn metadata(&self) -> Result<Option<Metadata>> {
        let lock_file = self.get_lock()?;
        lock_file
            .lock_shared()
            .context(DBError::FailedToLockFile(self.lock_file_path.to_path_buf()))?;

        self.metadata_unlocked()
    }

    /// Returns the metadata of the database without locking
    fn metadata_unlocked(&self) -> Result<Option<Metadata>> {
        let path = self.path.as_path();
        let file_path = path.join("metadata");

        if !file_path.is_file() {
            return Ok(None);
        }

        let data: Metadata = deserialize_file(file_path).context(DBError::FailedToReadMetadata)?;
        Ok(Some(data))
    }

    /// Returns the path of the database directory
    #[must_use]
    pub fn path(&self) -> &Path {
        self.path.as_path()
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
    ///
    /// ## Errors
    ///
    /// * [`DBError::FailedToOpenLockFile`]: could not open the lock file
    /// * [`DBError::FailedToLockFile`]: could not lock the lock file
    pub fn exists<T>(&self, id: &Id<T>) -> Result<bool>
    where
        T: AsTable,
    {
        self.exists_impl(T::name(), &id.to_string())
    }

    /// Checks if a record exists in the database
    fn exists_impl(&self, table_name: &str, id: &str) -> Result<bool> {
        let lock_file = self.get_lock()?;
        lock_file
            .lock_shared()
            .context(DBError::FailedToLockFile(self.lock_file_path.to_path_buf()))?;

        Ok(self.exists_impl_unlocked(table_name, id))
    }

    /// Checks if a record exists in the database without locking
    fn exists_impl_unlocked(&self, table_name: &str, id: &str) -> bool {
        let path = self.path.as_path();
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
    /// * [`DBError::FailedToOpenLockFile`]: Failed to open lock file
    /// * [`DBError::FailedToLockFile`]: Failed to lock file
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
        let lock_file = self.get_lock()?;
        lock_file
            .lock()
            .context(DBError::FailedToLockFile(self.lock_file_path.to_path_buf()))?;

        let id = record.get_id();

        if id.is_none() {
            return Err(DBError::InvalidKey(id.to_string()).into());
        }

        let meta = self
            .metadata_unlocked()
            .context(DBError::FailedToReadMetadata)?
            .context(DBError::NoMetadata)?;

        if meta.tables.is_empty() {
            return Err(DBError::NoTables.into());
        }

        for (field_name, ref_table, get_fk_id) in T::get_foreign_keys() {
            let fk_id_option = get_fk_id(record);
            if let Some(fk_id_str) = fk_id_option {
                if !self.exists_impl_unlocked(ref_table, fk_id_str) {
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
        let path = self.path.as_path();
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

        serialize_file(&file_path, record).context(DBError::FailedToSerializeFile(file_path))
    }

    /// Writes the metadata of the database
    fn write_metadata(&self, meta: &Metadata) -> Result<()> {
        let lock_file = self.get_lock()?;
        lock_file
            .lock()
            .context(DBError::FailedToLockFile(self.lock_file_path.to_path_buf()))?;

        let path = self.path.as_path();
        let file_path = path.join("metadata");

        serialize_file(file_path, meta).context(DBError::FailedToSerializeMetadata)?;
        Ok(())
    }
}

/// A builder for [Database]
#[derive(Debug, Default)]
pub struct DatabaseBuilder {
    params: Option<ArgonParams>,
    pass: Option<String>,
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
            params: None,
            pass: None,
            path: Some(path.to_path_buf()),
            tables: HashSet::new(),
        }
    }

    // ------------------------
    // START OF BUILDER METHODS
    // ------------------------

    /// Sets the Argon2 parameters to use for hashing
    ///
    /// Refer to [`ArgonParams`] for how to build an instance
    #[must_use]
    pub fn argon2_params(mut self, params: ArgonParams) -> Self {
        self.params = Some(params);
        self
    }

    /// Adds encryption to the database with the provided password
    ///
    /// ## Arguments
    ///
    /// * `pass` - The password to encrypt the database with
    #[must_use]
    pub fn encryption<S>(mut self, pass: S) -> Self
    where
        S: AsRef<str>,
    {
        self.pass = Some(pass.as_ref().to_string());
        self
    }

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

        let params = Arc::new(self.params);
        let mut db = Database {
            derived_key: Arc::new(None),
            lock_file_path: Arc::new(path.join(".minidb-lock")),
            path: Arc::new(path.clone()),
        };
        let meta =
            if let Some(meta) = Database::metadata(&db).context(DBError::FailedToReadMetadata)? {
                meta
            } else {
                let mut m = Metadata {
                    params: (*params).clone(),
                    salt: if self.pass.is_some() {
                        Some(generate_salt()?)
                    } else {
                        None
                    },
                    tables: self.tables,
                };

                db.derived_key = Arc::new(if let Some(pass) = &self.pass {
                    if let Some(salt) = &m.salt {
                        Some(derive_key((*params).clone(), pass, salt)?)
                    } else {
                        let salt = generate_salt()?;
                        m.salt = Some(salt);
                        Some(derive_key((*params).clone(), pass, salt)?)
                    }
                } else {
                    None
                });

                db.write_metadata(&m)
                    .context(DBError::FailedToWriteMetadata)?;
                m
            };

        meta.tables
            .par_iter()
            .map(|table| {
                // TODO consider if hashing the table name is worth it
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
    params: Option<ArgonParams>,
    salt: Option<Salt>,
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
