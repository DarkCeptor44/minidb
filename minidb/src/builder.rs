// Copyright (c) 2026, DarkCeptor44
//
// This file is licensed under the GNU Lesser General Public License
// (either version 3 or, at your option, any later version).
//
// This software comes without any warranty, express or implied. See the
// GNU Lesser General Public License for details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with this software. If not, see <https://www.gnu.org/licenses/>.

use std::{fmt::Debug, path::PathBuf};

use crate::encryption::derive_key_from_password;
use crate::model::TableModel;
use crate::{ArgonKey, META_TABLE, MiniDB, SETTINGS_TABLE};
use anyhow::{Context, Result};
use chacha20poly1305::KeyInit;
use chacha20poly1305::XChaCha20Poly1305;
use redb::{Database, WriteTransaction};

type Initializer = Box<dyn Fn(&WriteTransaction) -> Result<()>>;

/// A builder for a [`MiniDB`]
///
/// ## Example
///
/// ```rust,ignore
/// use minidb::MiniDBBuilder;
///
/// // create the database with the path to the database file. The file can have any extension
/// // but it's recommended to use `.redb` so you can differentiate
/// // between a MiniDB/redb database and a SQLite/other embedded database
/// let db = MiniDBBuilder::new("test.redb")
///     .table::<Person>() // you must register all table models
///     .table::<Car>()
///     .build()
///     .unwrap();
/// ```
pub struct MiniDBBuilder {
    path: PathBuf,
    initializers: Vec<Initializer>,
    key_source: Option<KeySource>,
}

impl Debug for MiniDBBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MiniDBBuilder")
            .field("path", &self.path)
            .field("key_source", &self.key_source)
            .finish_non_exhaustive()
    }
}

impl MiniDBBuilder {
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
    pub fn new<P>(path: P) -> Self
    where
        P: Into<PathBuf>,
    {
        Self {
            path: path.into(),
            initializers: Vec::new(),
            key_source: None,
        }
    }

    /// Registers a table model
    ///
    /// ## Arguments
    ///
    /// * `T` - The table model to register
    ///
    /// ## Returns
    ///
    /// A new [`MiniDBBuilder`]
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// use minidb::{MiniDB, Table};
    ///
    /// #[derive(Table)]
    /// struct Person{
    ///     #[key]
    ///     id: String,
    /// }
    ///
    /// // create a MiniDB builder with the file path
    /// // and register the table Person
    /// let db = MiniDB::builder("test.redb")
    ///     .table::<Person>();
    /// ```
    #[must_use]
    pub fn table<T>(mut self) -> Self
    where
        T: TableModel + 'static,
    {
        self.initializers.push(Box::new(|txn| {
            txn.open_table(T::TABLE)
                .map(|_| ())
                .context("failed to init table")?;
            Ok(())
        }));
        self
    }

    /// Sets the key source
    ///
    /// ## Arguments
    ///
    /// * `source` - The key source, can be a password, a pre-derived key, or a function that returns a key (`[u8; 32]`)
    ///
    /// ## Returns
    ///
    /// A new [`MiniDBBuilder`]
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// use minidb::{KeySource, MiniDB};
    ///
    /// // create a MiniDB builder with a password
    /// let db = MiniDB::builder("test.redb")
    ///     // skipping table registering for convenience
    ///     .key_source(KeySource::Password("secretpassword".to_string()));
    ///
    /// // create a MiniDB builder with a pre-derived key
    /// let key = [1u8; 32];
    /// let db = MiniDB::builder("test.redb")
    ///     // skipping table registering for convenience
    ///     .key_source(KeySource::PreDerived(key));
    ///
    /// // create a MiniDB builder with a function that returns a key
    /// fn key_provider() -> [u8; 32] {
    ///     [1u8; 32]
    /// }
    ///
    /// let db = MiniDB::builder("test.redb")
    ///     // skipping table registering for convenience
    ///     .key_source(KeySource::ExternalKeyProvider(Box::new(key_provider)));
    /// ```
    #[must_use]
    pub fn key_source(mut self, source: KeySource) -> Self {
        self.key_source = Some(source);
        self
    }

    /// Builds the [`MiniDB`] from the builder
    ///
    /// ## Returns
    ///
    /// A new [`MiniDB`]
    ///
    /// ## Errors
    ///
    /// Returns an error if the database file already exists, if the bootstrap transaction fails, or if the key derivation fails
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// use minidb::{KeySource, MiniDB};
    ///
    /// // create a MiniDB
    /// let db = MiniDB::builder("test.redb")
    ///     // skipping table registering for convenience
    ///     .key_source(KeySource::Password("secretpassword".to_string())) // if you want the database to be encrypted
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn build(self) -> Result<MiniDB> {
        let db = Database::builder()
            .create(&self.path)
            .context("failed to create database file")?;

        let txn = db.begin_write().context("failed to begin bootstrap txn")?;
        {
            let _ = txn
                .open_table(META_TABLE)
                .context("failed to init meta table")?;
            let _ = txn
                .open_table(SETTINGS_TABLE)
                .context("failed to init settings table")?;
        }
        for init in self.initializers {
            init(&txn)?;
        }
        txn.commit().context("failed to commit bootstrap")?;

        let mut store = MiniDB::new(db);

        if let Some(source) = self.key_source {
            let key = match source {
                KeySource::Password(pass) => {
                    let salt = store
                        .get_salt()
                        .context("failed to get salt from meta table")?;

                    derive_key_from_password(&pass, Some(salt), None)
                        .context("failed to derive key")?
                }

                KeySource::PreDerived(key) => key,

                KeySource::ExternalKeyProvider(provider_fn) => provider_fn(),
            };

            store.set_cipher(XChaCha20Poly1305::new(&key.into()));
        }

        Ok(store)
    }
}

/// The key source
///
/// ## Variants
///
/// * `KeySource::Password(String)` - The key source is a password
/// * `KeySource::PreDerived(ArgonKey)` - The key source is a pre-derived key (`[u8; 32]`)
/// * `KeySource::ExternalKeyProvider(Box<dyn Fn() -> ArgonKey>)` - The key source is a function that returns a key (`[u8; 32]`)
pub enum KeySource {
    /// The key source is a password
    Password(String),

    /// The key source is a pre-derived key (`[u8; 32]`)
    PreDerived(ArgonKey),

    /// The key source is a function that returns a key (`[u8; 32]`)
    ExternalKeyProvider(Box<dyn Fn() -> ArgonKey>),
}

impl Debug for KeySource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeySource::Password(_) => f.debug_tuple("Password").field(&"********").finish(),
            KeySource::PreDerived(key) => f.debug_tuple("PreDerived").field(key).finish(),
            KeySource::ExternalKeyProvider(_) => f
                .debug_tuple("ExternalKeyProvider")
                .field(&"<closure>")
                .finish(),
        }
    }
}
