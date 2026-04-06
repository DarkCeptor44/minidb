// This Source Code Form is subject to the terms of the
// Mozilla Public License, v. 2.0. If a copy of the MPL was not distributed
// with this file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![forbid(unsafe_code)]
#![warn(clippy::pedantic, missing_docs, missing_debug_implementations)]
#![allow(clippy::doc_markdown)]

//! # MiniDB
//!
//! A minimal-but-functional structured wrapper for [redb] with serialization/deserialization powered by [Postcard](postcard) and [Serde](serde).
//!
//! ## Key Features
//!
//! * ACID compliant and whatever else [redb] claims
//! * Automatic serialization/deserialization with [postcard], using [serde]
//! * Structured key-value storage with automatic [CUID2](cuid2) IDs
//! * Type-safe operations (mostly)
//! * Optional encryption using [XChaCha20Poly1305]
//! * Includes derive macros (e.g., `#[derive(Table)]`) for easy table definition
//! * Re-exports [serde] for convenience
//! * Re-exports [redb] and some direct/less-opinionated methods for advanced usage
//! * "Relational" (requires manual management of foreign keys)
//!
//! ## MSRV
//!
//! | Version | MSRV | Edition |
//! | --- | --- | --- |
//! | 0.1.x - 0.2.x | 1.89 | 2024 |
//!
//! ## Getting Started
//!
//! Add `minidb` to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! minidb = { version = "0.2.0", features = ["macros"] } # Or just "0.2.0" if you don't need the macros
//! ```
//!
//! ## Basic Usage Example
//!
//! This example demonstrates defining a table using the [Table] derive macro, creating a database instance, inserting an item, and retrieving it.
//!
//! **Note**: This example requires the `macros` feature.
//!
//! ```rust,no_run
//! use minidb::{
//!     serde::{Deserialize, Serialize},
//!     MiniDB, Table
//! };
//!
//! #[derive(Table, Serialize, Deserialize, Debug)]
//! #[serde(crate = "minidb::serde")] // required if using re-exported serde
//! #[minidb(name = "people")]
//! struct Person {
//!     #[key]
//!     id: String,
//!     name: String,
//!     age: u8,
//! }
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // create/open the database, registering the `Person` table
//!     let db = MiniDB::builder("path/to/my_db.redb")
//!         .table::<Person>()
//!         .build()?;
//!
//!     // create a new Person (ID will be generated automatically so the field should be empty)
//!     let mut person_to_insert = Person {
//!         id: String::new(),
//!         name: "John Doe".to_string(),
//!         age: 31,
//!     };
//!
//!     // insert the person into the database
//!     db.insert(&mut person_to_insert)?;
//!
//!     // retrieve the person by ID
//!     let retrieved_person: Option<Person> = db.get(&person_to_insert.id)?;
//!
//!     match retrieved_person {
//!         Some(p) => println!("Retrieved person: {:?}", p),
//!         None => println!("Person not found"),
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! See the individual module and item documentation for further details on specific components.
//!
//! ## License
//!
//! Licensed under the Mozilla Public License 2.0 ([MPL-2.0](https://www.mozilla.org/en-US/MPL/2.0/)).

mod builder;
mod encryption;
mod error;
mod model;
mod testing;
mod transaction;

pub use crate::{
    builder::{KeySource, MiniDBBuilder},
    error::Error,
    model::{Table, TableIterator},
    transaction::Transaction,
};
#[cfg(feature = "macros")]
pub use minidb_macros::Table;
pub use redb;
pub use serde;

use std::{fmt::Debug, path::PathBuf};

use crate::{
    encryption::{decrypt_bytes, encrypt_bytes},
    error::Result,
};
use argon2::password_hash::{SaltString, rand_core::OsRng};
use chacha20poly1305::XChaCha20Poly1305;
use redb::{Database, ReadableDatabase, ReadableTable, ReadableTableMetadata, TableDefinition};
use serde::{Deserialize, Serialize};

pub(crate) const META_TABLE: TableDefinition<&'static str, &[u8]> = TableDefinition::new("meta");
pub(crate) const SETTINGS_TABLE: TableDefinition<&'static str, &[u8]> =
    TableDefinition::new("settings");

const META_KEY_SALT: &str = "salt";

pub(crate) type ArgonKey = [u8; 32];

/// A MiniDB
///
/// This is a wrapper around [`redb::Database`], but also stores the [`XChaCha20Poly1305`] instance to handle the optional encryption
pub struct MiniDB {
    db: Database,
    cipher: Option<XChaCha20Poly1305>,
}

impl Debug for MiniDB {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MiniDB")
            .field("db", &self.db)
            .finish_non_exhaustive()
    }
}

impl MiniDB {
    /// Creates a new [`MiniDBBuilder`]
    ///
    /// ## Arguments
    ///
    /// * `path` - The path to the database file
    ///
    /// ## Returns
    ///
    /// A new [`MiniDBBuilder`]
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// use minidb::MiniDB;
    ///
    /// // create a MiniDB builder with the file path
    /// let db = MiniDB::builder("test.redb");
    /// ```
    pub fn builder<P>(path: P) -> MiniDBBuilder
    where
        P: Into<PathBuf>,
    {
        MiniDBBuilder::new(path)
    }

    /// Creates a new [`MiniDB`] from a [`redb::Database`]
    ///
    /// If you don't need advanced features then I recommend [`MiniDB::builder`], you can pass the tables to it, the path, and whether or not to use encryption.
    ///
    /// ## Arguments
    ///
    /// * `db` - The [`redb::Database`] to use
    ///
    /// ## Returns
    ///
    /// A new [`MiniDB`] with the provided [`redb::Database`]
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// use minidb::MiniDB;
    ///
    /// let db = MiniDB::new(redb::Database::open("test.redb").unwrap());
    /// ```
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self { db, cipher: None }
    }

    /// Sets the [`XChaCha20Poly1305`] instance to use, this implies encryption if [Some]
    ///
    /// I recommend using [`MiniDB::builder`] and setting the encryption with [`MiniDBBuilder::key_source`] instead
    ///
    /// ## Arguments
    ///
    /// * `cipher` - The [`XChaCha20Poly1305`] instance to use
    pub fn set_cipher(&mut self, cipher: XChaCha20Poly1305) {
        self.cipher = Some(cipher);
    }

    // EMD OF BUILDERS

    /// Retrieves all items from a table
    ///
    /// ## Arguments
    ///
    /// * `T` - The table model
    ///
    /// ## Returns
    ///
    /// A [`Result`] containing the vector of all items in the table `T`
    ///
    /// ## Errors
    ///
    /// Returns an error if the table is not found, if the table is not initialized, or if the decryption/deserialization fails
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// // retrieves all items from the table `Person`
    /// let people = db.all::<Person>().unwrap();
    ///
    /// // or
    ///
    /// let people: Vec<Person> = db.all().unwrap();
    /// ```
    pub fn all<T>(&self) -> Result<Vec<T>>
    where
        T: Table,
    {
        let txn = self.db.begin_read()?;
        let table = txn.open_table(T::TABLE)?;

        let mut results = Vec::new();
        for item in table.iter()? {
            let (_key, value) = item?;

            let decoded: T = if let Some(cipher) = &self.cipher {
                let decrypted = decrypt_bytes(cipher, value.value())?;
                postcard::from_bytes(&decrypted)?
            } else {
                postcard::from_bytes(value.value())?
            };

            results.push(decoded);
        }

        Ok(results)
    }

    /// Force a check of the integrity of the database file, and repair it if possible.
    ///
    /// Note: Calling this function is unnecessary during normal operation. redb will automatically
    /// detect and recover from crashes, power loss, and other unclean shutdowns. This function is
    /// quite slow and should only be used when you suspect the database file may have been modified
    /// externally to redb, or that a redb bug may have left the database in a corrupted state.
    ///
    /// ## Returns
    ///
    /// `Ok(true)` if the database passed integrity checks, `Ok(false)` if it failed but was repaired,
    /// and `Err(Corrupted)` if the check failed and the file could not be repaired
    ///
    /// ## Errors
    ///
    /// Returns an error if the check fails and the file could not be repaired
    pub fn check_integrity(&mut self) -> Result<bool> {
        Ok(self.db.check_integrity()?)
    }

    /// Compacts the database file
    ///
    /// ## Returns
    ///
    /// `Ok(true)` if compacting was performed, and `Ok(false)` if no further compacting was possible
    ///
    /// ## Errors
    ///
    /// Returns an error if the compacting fails
    pub fn compact(&mut self) -> Result<bool> {
        Ok(self.db.compact()?)
    }

    /// Creates the table if it doesn't exist
    ///
    /// Recommended to use [`MiniDBBuilder::table`] instead.
    ///
    /// ## Arguments
    ///
    /// * `T` - The table model
    ///
    /// ## Errors
    ///
    /// Returns an error if the table creation fails
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// db.create_table::<Person>().unwrap();
    /// ```
    pub fn create_table<T>(&self) -> Result<()>
    where
        T: Table,
    {
        self.create_table_impl(T::TABLE)
    }

    /// Creates the table if it doesn't exist
    ///
    /// Use [`MiniDB::create_table`] instead
    pub(crate) fn create_table_impl<K, V>(&self, table: TableDefinition<K, V>) -> Result<()>
    where
        K: redb::Key,
        V: redb::Value,
    {
        let txn = self.db.begin_write()?;
        {
            let _ = txn
                .open_table(table)
                .map_err(|e| Error::TableInitialization {
                    name: table.to_string(),
                    source: e,
                })?;
        }
        txn.commit()?;
        Ok(())
    }

    /// Exports a table as a JSON string
    ///
    /// ## Arguments
    ///
    /// * `T` - The table model
    /// * `pretty` - Whether to pretty print the JSON
    ///
    /// ## Returns
    ///
    /// * `String` - The JSON string
    ///
    /// ## Errors
    ///
    /// Returns an error if the table is not found, if the table is not initialized, or if the encryption/serialization fails
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// let json = db.export_table::<Person>(true).unwrap();
    /// ```
    pub fn export_table<T>(&self, pretty: bool) -> Result<String>
    where
        T: Table,
    {
        let all_items: Vec<T> = self.all()?;
        let json = if pretty {
            serde_json::to_string_pretty(&all_items)
        } else {
            serde_json::to_string(&all_items)
        }?;
        Ok(json)
    }

    /// Iterates over all items in a table and applies a function to each item
    ///
    /// ## Arguments
    ///
    /// * `T` - The table model
    /// * `f` - The function to apply to each item
    ///
    /// ## Errors
    ///
    /// Returns an error if the table is not found, if the table is not initialized, or if the decryption/deserialization fails
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// db.for_each::<Person>(|person| {
    ///     println!("Person name: {}", person.name);
    /// }).unwrap();
    /// ```
    pub fn for_each<T, F>(&self, mut f: F) -> Result<()>
    where
        T: Table,
        F: FnMut(&T),
    {
        let txn = self.db.begin_read()?;
        let table = txn.open_table(T::TABLE)?;

        for item in table.iter()? {
            let (_, value) = item?;

            let data: T = if let Some(cipher) = &self.cipher {
                let decrypted = decrypt_bytes(cipher, value.value())?;
                postcard::from_bytes(&decrypted)?
            } else {
                postcard::from_bytes(value.value())?
            };

            f(&data);
        }

        Ok(())
    }

    /// Inserts an item into a table
    ///
    /// ## Arguments
    ///
    /// * `T` - The table model
    /// * `item` - The item to insert
    ///
    /// ## Errors
    ///
    /// Returns an error if the table is not found, if the table is not initialized, or if the encryption/serialization fails
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// let mut person = Person {
    ///     id: String::new(), // ID must be new so it can be generated automatically
    ///     name: "John Doe".to_string(),
    ///     age: 31,
    /// };
    ///
    /// db.insert(&mut person).unwrap();
    /// ```
    pub fn insert<T>(&self, item: &mut T) -> Result<()>
    where
        T: Table,
    {
        if item.get_id().trim().is_empty() {
            let id = cuid2::slug();
            item.set_id(id);
        }

        let txn = self.db.begin_write()?;
        {
            let mut table = txn.open_table(T::TABLE)?;
            let bytes = postcard::to_stdvec(item)?;

            let to_write: Vec<u8> = if let Some(cipher) = &self.cipher {
                encrypt_bytes(cipher, &bytes)?
            } else {
                bytes
            };

            table.insert(item.get_id(), to_write.as_slice())?;
        }
        txn.commit()?;
        Ok(())
    }

    /// Inserts multiple items into a table
    ///
    /// ## Arguments
    ///
    /// * `T` - The table model
    /// * `items` - The items to insert
    ///
    /// ## Errors
    ///
    /// Returns an error if the table is not found, if the table is not initialized, or if the encryption/serialization fails
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// let mut people = vec![
    ///     Person {
    ///         id: String::new(),
    ///         name: "John Doe".to_string(),
    ///         age: 31,
    ///     },
    ///
    ///     // ...
    /// ];
    ///
    /// db.insert_many(&mut people).unwrap();
    /// ```
    pub fn insert_many<T>(&self, items: &mut [T]) -> Result<()>
    where
        T: Table,
    {
        let txn = self.db.begin_write()?;
        {
            let mut table = txn.open_table(T::TABLE)?;
            for item in items {
                if item.get_id().trim().is_empty() {
                    let id = cuid2::slug();
                    item.set_id(id);
                }

                let bytes = postcard::to_stdvec(&item)?;
                let to_write: Vec<u8> = if let Some(cipher) = &self.cipher {
                    encrypt_bytes(cipher, &bytes)?
                } else {
                    bytes
                };

                table.insert(item.get_id(), to_write.as_slice())?;
            }
        }
        txn.commit()?;
        Ok(())
    }

    /// Checks if a table is empty
    ///
    /// ## Arguments
    ///
    /// * `T` - The table model
    ///
    /// ## Returns
    ///
    /// * `Ok(true)` if the table is empty
    /// * `Ok(false)` if the table is not empty
    ///
    /// ## Errors
    ///
    /// Returns an error if the table is not found or couldn't be opened
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// let is_empty = db.is_empty::<Person>().unwrap();
    /// println!("Is the table empty? {}", is_empty);
    /// ```
    pub fn is_empty<T>(&self) -> Result<bool>
    where
        T: Table,
    {
        let txn = self.db.begin_read()?;
        let table = txn.open_table(T::TABLE)?;
        Ok(table.is_empty()?)
    }

    /// Retrieves an item from a table
    ///
    /// ## Arguments
    ///
    /// * `T` - The table model
    /// * `id` - The id of the item to retrieve
    ///
    /// ## Returns
    ///
    /// * `Ok(Some(item))` if the item was found
    /// * `Ok(None)` if the item was not found
    ///
    /// ## Errors
    ///
    /// Returns an error if the table is not found, if the table is not initialized, or if the encryption/serialization fails
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// // retrieves an item from the table `Person`
    /// let person = db.get::<Person>("person_id").unwrap().unwrap();
    /// println!("Person name: {}", person.name);
    ///
    /// // or
    ///
    /// let person: Option<Person> = db.get("person_id").unwrap();
    /// if let Some(person) = person {
    ///     println!("Person name: {}", person.name);
    /// }
    /// ```
    pub fn get<T>(&self, id: &str) -> Result<Option<T>>
    where
        T: Table,
    {
        let txn = self.db.begin_read()?;
        let table = txn.open_table(T::TABLE)?;
        let value = table.get(id)?;

        let Some(bytes) = value else {
            return Ok(None);
        };

        let item: T = if let Some(cipher) = &self.cipher {
            let decrypted = decrypt_bytes(cipher, bytes.value())?;
            postcard::from_bytes(&decrypted)?
        } else {
            postcard::from_bytes(bytes.value())?
        };

        Ok(Some(item))
    }

    /// Retrieves the salt from the meta table
    pub(crate) fn get_salt(&self) -> Result<String> {
        let value: Option<String> = self.get_meta(META_KEY_SALT)?;

        if let Some(salt) = value {
            Ok(salt)
        } else {
            let new_salt_string = SaltString::generate(&mut OsRng);
            self.set_meta(META_KEY_SALT, &new_salt_string.to_string())?;
            Ok(new_salt_string.to_string())
        }
    }

    /// Retrieves an item from the meta table
    pub(crate) fn get_meta<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: for<'a> Deserialize<'a>,
    {
        let txn = self.db.begin_read()?;
        let table = txn.open_table(META_TABLE)?;
        let value = table.get(key)?;

        if let Some(bytes) = value {
            let item: T = postcard::from_bytes(bytes.value())?;
            Ok(Some(item))
        } else {
            Ok(None)
        }
    }

    /// Retrieves an item from the settings table
    ///
    /// ## Arguments
    ///
    /// * `T` - The table model
    /// * `key` - The key of the item to retrieve
    ///
    /// ## Returns
    ///
    /// * `Ok(Some(item))` if the item was found
    /// * `Ok(None)` if the item was not found
    ///
    /// ## Errors
    ///
    /// Returns an error if the table is not found, if the table is not initialized, or if the encryption/serialization fails
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// let setting = db.get_setting::<String>("key").unwrap().unwrap();
    ///
    /// // or
    ///
    /// let setting: Option<String> = db.get_setting("key").unwrap();
    /// ```
    pub fn get_setting<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: for<'a> Deserialize<'a>,
    {
        let txn = self.db.begin_read()?;
        let table = txn.open_table(SETTINGS_TABLE)?;
        let value = table.get(key)?;

        let Some(bytes) = value else {
            return Ok(None);
        };

        let item: T = if let Some(cipher) = &self.cipher {
            let decrypted = decrypt_bytes(cipher, bytes.value())?;
            postcard::from_bytes(&decrypted)?
        } else {
            postcard::from_bytes(bytes.value())?
        };

        Ok(Some(item))
    }

    /// Removes an item from the table
    ///
    /// ## Arguments
    ///
    /// * `T` - The table model
    /// * `key` - The key of the item to remove
    ///
    /// ## Returns
    ///
    /// * `Ok(Some(item))` if the item was removed
    /// * `Ok(None)` if the item was not found
    ///
    /// ## Errors
    ///
    /// Returns an error if the table is not found, if the table is not initialized, or if the encryption/serialization fails
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// let removed_person = db.remove::<Person>("person_id").unwrap().unwrap();
    ///
    /// // or
    ///
    /// let removed_person: Option<Person> = db.remove("person_id").unwrap();
    /// if let Some(person) = removed_person {
    ///     println!("Person name: {}", person.name);
    /// }
    /// ```
    pub fn remove<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: Table,
    {
        let txn = self.db.begin_write()?;
        let mut result = None;
        {
            let mut table = txn.open_table(T::TABLE)?;
            let maybe_bytes = table.remove(key)?;

            if let Some(bytes) = maybe_bytes {
                let item: T = if let Some(cipher) = &self.cipher {
                    let decrypted = decrypt_bytes(cipher, bytes.value())?;
                    postcard::from_bytes(&decrypted)?
                } else {
                    postcard::from_bytes(bytes.value())?
                };

                result = Some(item);
            }
        }
        txn.commit()?;
        Ok(result)
    }

    /// Removes multiple items from the table
    ///
    /// ## Arguments
    ///
    /// * `T` - The table model
    /// * `keys` - The keys of the items to remove
    ///
    /// ## Returns
    ///
    /// * `Vec<T>` - The items that were removed
    ///
    /// ## Errors
    ///
    /// Returns an error if the table is not found, if the table is not initialized, or if the encryption/serialization fails
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// let removed_people = db.remove_many::<Person>(&["person_id1", "person_id2", ...]).unwrap();
    ///
    /// // or
    ///
    /// let removed_people: Vec<Person> = db.remove_many(&["person_id1", "person_id2", ...]).unwrap();
    /// ```
    pub fn remove_many<T>(&self, keys: &[&str]) -> Result<Vec<T>>
    where
        T: Table,
    {
        let txn = self.db.begin_write()?;
        let mut result = Vec::new();
        {
            let mut table = txn.open_table(T::TABLE)?;
            for key in keys {
                let maybe_bytes = table.remove(key)?;

                if let Some(bytes) = maybe_bytes {
                    let item: T = if let Some(cipher) = &self.cipher {
                        let decrypted = decrypt_bytes(cipher, bytes.value())?;
                        postcard::from_bytes(&decrypted)?
                    } else {
                        postcard::from_bytes(bytes.value())?
                    };

                    result.push(item);
                }
            }
        }
        txn.commit()?;
        Ok(result)
    }

    /// Sets an item in the meta table
    pub(crate) fn set_meta<T>(&self, key: &str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        let txn = self.db.begin_write()?;
        {
            let mut table = txn.open_table(META_TABLE)?;
            let bytes = postcard::to_stdvec(value)?;
            table.insert(key, bytes.as_slice())?;
        }
        txn.commit()?;
        Ok(())
    }

    /// Sets an item in the settings table
    ///
    /// ## Arguments
    ///
    /// * `T` - The table model
    /// * `key` - The key of the item to set
    /// * `value` - The value of the item to set
    ///
    /// ## Errors
    ///
    /// Returns an error if the table is not found, if the table is not initialized, or if the encryption/serialization fails
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// db.set_setting("key", "value").unwrap();
    /// ```
    pub fn set_setting<T>(&self, key: &str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        let txn = self.db.begin_write()?;
        {
            let mut table = txn.open_table(SETTINGS_TABLE)?;
            let bytes = postcard::to_stdvec(value)?;

            let to_write: Vec<u8> = if let Some(cipher) = &self.cipher {
                encrypt_bytes(cipher, &bytes)?
            } else {
                bytes
            };

            table.insert(key, to_write.as_slice())?;
        }
        txn.commit()?;
        Ok(())
    }

    /// Starts a write transaction.
    ///
    /// This allows grouping multiple operations (insert, update, remove) into a single atomic transaction.
    ///
    /// ## Errors
    ///
    /// Returns an error if the transaction fails to begin, if the closure returns an error, or if the commit fails.
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// db.transaction(|txn| {
    ///     txn.insert(&mut user1)?;
    ///     txn.update(&mut user2)?;
    ///     txn.remove("user3_id")?;
    ///     Ok(())
    /// }).unwrap();
    /// ```
    pub fn transaction<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&Transaction) -> Result<R>,
    {
        let txn = self.db.begin_write()?;
        let transaction = Transaction {
            txn,
            cipher: self.cipher.as_ref(),
        };

        let result = f(&transaction)?;
        transaction.txn.commit()?;
        Ok(result)
    }

    /// Updates an item in the table
    ///
    /// ## Arguments
    ///
    /// * `T` - The table model
    /// * `item` - The item to update
    ///
    /// ## Errors
    ///
    /// Returns an error if the table is not found, if the table is not initialized, or if the encryption/serialization fails
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// let mut person = Person {
    ///     id: "person_id".to_string(),
    ///     name: "John Doe".to_string(),
    ///     age: 31,
    /// };
    ///
    /// person.age = 32;
    ///
    /// db.update(&person).unwrap();
    /// ```
    pub fn update<T>(&self, item: &T) -> Result<()>
    where
        T: Table,
    {
        if item.get_id().trim().is_empty() {
            return Err(Error::EmptyID);
        }

        let txn = self.db.begin_write()?;
        {
            let mut table = txn.open_table(T::TABLE)?;
            let bytes = postcard::to_stdvec(&item)?;

            let to_write: Vec<u8> = if let Some(cipher) = &self.cipher {
                encrypt_bytes(cipher, &bytes)?
            } else {
                bytes
            };

            table.insert(item.get_id(), to_write.as_slice())?;
        }
        txn.commit()?;
        Ok(())
    }

    /// Updates multiple items in the table
    ///
    /// ## Arguments
    ///
    /// * `T` - The table model
    /// * `items` - The items to update
    ///
    /// ## Errors
    ///
    /// Returns an error if the table is not found, if the table is not initialized, or if the encryption/serialization fails
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// let mut people = vec![
    ///     Person {
    ///         id: "person_id".to_string(),
    ///         name: "John Doe".to_string(),
    ///         age: 31,
    ///     },
    ///
    ///     // ...
    /// ];
    ///
    /// for person in &mut people {
    ///     person.age = 32;
    /// }
    ///
    /// db.update_many(&mut people).unwrap();
    /// ```
    pub fn update_many<T>(&self, items: &[T]) -> Result<()>
    where
        T: Table,
    {
        let txn = self.db.begin_write()?;
        {
            let mut table = txn.open_table(T::TABLE)?;
            for item in items {
                if item.get_id().trim().is_empty() {
                    return Err(Error::EmptyID);
                }
                let bytes = postcard::to_stdvec(item)?;

                let to_write: Vec<u8> = if let Some(cipher) = &self.cipher {
                    encrypt_bytes(cipher, &bytes)?
                } else {
                    bytes
                };

                table.insert(item.get_id(), to_write.as_slice())?;
            }
        }
        txn.commit()?;
        Ok(())
    }

    /// Returns an iterator over all items in a table, allowing for custom processing
    ///
    /// ## Arguments
    ///
    /// * `T` - The table model
    /// * `f` - The function that processes the iterator
    ///
    /// ## Returns
    ///
    /// A [`Result`] containing the result of the processing function `f`
    ///
    /// ## Errors
    ///
    /// Returns an error if the table is not found, if the table is not initialized, or if the encryption/serialization fails
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// let first_five: Vec<Person> = db.view_all::<Person, _, _>(|iter| {
    ///     iter.take(5).collect::<Result<Vec<_>>>().unwrap()
    /// }).unwrap();
    /// ```
    pub fn view_all<T, F, R>(&self, f: F) -> Result<R>
    where
        T: Table,
        F: FnOnce(TableIterator<'_, T>) -> R,
    {
        let txn = self.db.begin_read()?;
        let table = txn.open_table(T::TABLE)?;
        let mut iter = TableIterator::new(table.iter()?);

        if let Some(cipher) = &self.cipher {
            iter = iter.with_cipher(cipher);
        }

        Ok(f(iter))
    }
}
