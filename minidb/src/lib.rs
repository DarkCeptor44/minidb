mod encryption;
mod testing;

pub use redb;

use std::path::PathBuf;

use crate::encryption::{decrypt_bytes, derive_key_from_password, encrypt_bytes};
use anyhow::{Context, Result, anyhow};
use argon2::password_hash::{SaltString, rand_core::OsRng};
use chacha20poly1305::{KeyInit, XChaCha20Poly1305};
use redb::{
    Database, ReadableDatabase, ReadableTable, ReadableTableMetadata, TableDefinition,
    WriteTransaction,
};
use serde::{Deserialize, Serialize};

const META_TABLE: TableDefinition<&'static str, &[u8]> = TableDefinition::new("meta");
const SETTINGS_TABLE: TableDefinition<&'static str, &[u8]> = TableDefinition::new("settings");

const META_KEY_SALT: &str = "salt";

type Initializer = Box<dyn Fn(&WriteTransaction) -> Result<()>>;
type ArgonKey = [u8; 32];

pub trait TableModel: Serialize + for<'de> Deserialize<'de> {
    const TABLE: TableDefinition<'_, &'static str, &[u8]>;

    fn get_id(&self) -> &str;
    fn set_id(&mut self, id: String);
}

pub struct MiniDBBuilder {
    path: PathBuf,
    initializers: Vec<Initializer>,
    key_source: Option<KeySource>,
}

impl MiniDBBuilder {
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

    pub fn key_source(mut self, source: KeySource) -> Self {
        self.key_source = Some(source);
        self
    }

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

        let mut store = MiniDB { db, cipher: None };

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

            store.cipher = Some(XChaCha20Poly1305::new(&key.into()));
        }

        Ok(store)
    }
}

pub enum KeySource {
    Password(String),
    PreDerived(ArgonKey),
    ExternalKeyProvider(Box<dyn Fn() -> ArgonKey>),
}

pub struct MiniDB {
    db: Database,
    cipher: Option<XChaCha20Poly1305>,
}

impl MiniDB {
    pub fn builder<P>(path: P) -> MiniDBBuilder
    where
        P: Into<PathBuf>,
    {
        MiniDBBuilder::new(path)
    }

    // EMD OF BUILDERS

    pub fn all<T>(&self) -> Result<Vec<T>>
    where
        T: TableModel,
    {
        let txn = self.db.begin_read().context("failed to begin read")?;
        let table = txn.open_table(T::TABLE).context("failed to open table")?;

        let mut results = Vec::new();
        for item in table.iter()? {
            let (_key, value) = item?;

            let decoded: T = if let Some(cipher) = &self.cipher {
                let decrypted =
                    decrypt_bytes(cipher, value.value()).context("failed to decrypt bytes")?;
                postcard::from_bytes(&decrypted).context("failed to deserialize from postcard")?
            } else {
                postcard::from_bytes(value.value())
                    .context("failed to deserialize from postcard")?
            };

            results.push(decoded);
        }

        Ok(results)
    }

    pub fn create_table<T>(&self) -> Result<()>
    where
        T: TableModel,
    {
        self.create_table_impl(T::TABLE)
    }

    fn create_table_impl<K, V>(&self, table: TableDefinition<K, V>) -> Result<()>
    where
        K: redb::Key,
        V: redb::Value,
    {
        let txn = self.db.begin_write().context("failed to begin write")?;
        {
            let _ = txn.open_table(table).context("failed to open table")?;
        }
        txn.commit().context("failed to commit write")?;
        Ok(())
    }

    pub fn for_each<T, F>(&self, mut f: F) -> Result<()>
    where
        T: TableModel,
        F: FnMut(T),
    {
        let txn = self.db.begin_read().context("failed to begin read")?;
        let table = txn.open_table(T::TABLE)?;

        for item in table.iter()? {
            let (_, value) = item?;

            let data: T = if let Some(cipher) = &self.cipher {
                let decrypted =
                    decrypt_bytes(cipher, value.value()).context("failed to decrypt data")?;
                postcard::from_bytes(&decrypted).context("failed to deserialize postcard")?
            } else {
                postcard::from_bytes(value.value()).context("failed to deserialize postcard")?
            };

            f(data);
        }

        Ok(())
    }

    pub fn insert<T>(&self, item: &mut T) -> Result<()>
    where
        T: TableModel,
    {
        if item.get_id().trim().is_empty() {
            let id = cuid2::slug();
            item.set_id(id);
        }

        let txn = self.db.begin_write().context("failed to begin write")?;
        {
            let mut table = txn.open_table(T::TABLE).context("failed to open table")?;
            let bytes = postcard::to_stdvec(item).context("failed to serialize to postcard")?;

            let to_write: Vec<u8> = if let Some(cipher) = &self.cipher {
                encrypt_bytes(cipher, &bytes).context("failed to encrypt bytes")?
            } else {
                bytes
            };

            table
                .insert(item.get_id(), to_write.as_slice())
                .context("failed to insert into table")?;
        }
        txn.commit().context("failed to commit to database")?;
        Ok(())
    }

    pub fn insert_many<T>(&self, items: &mut [T]) -> Result<()>
    where
        T: TableModel,
    {
        let txn = self.db.begin_write().context("failed to begin write")?;
        {
            let mut table = txn.open_table(T::TABLE).context("failed to open table")?;
            for item in items {
                if item.get_id().trim().is_empty() {
                    let id = cuid2::slug();
                    item.set_id(id);
                }

                let bytes =
                    postcard::to_stdvec(&item).context("failed to serialize to postcard")?;

                let to_write: Vec<u8> = if let Some(cipher) = &self.cipher {
                    encrypt_bytes(cipher, &bytes).context("failed to encrypt bytes")?
                } else {
                    bytes
                };

                table
                    .insert(item.get_id(), to_write.as_slice())
                    .context("failed to insert into table")?;
            }
        }
        txn.commit().context("failed to commit to database")?;
        Ok(())
    }

    pub fn is_empty<T>(&self) -> Result<bool>
    where
        T: TableModel,
    {
        let txn = self.db.begin_read().context("failed to begin read")?;
        let table = txn.open_table(T::TABLE).context("failed to open table")?;
        table
            .is_empty()
            .context("failed to check if table is empty")
    }

    pub fn get<T>(&self, id: &str) -> Result<Option<T>>
    where
        T: TableModel,
    {
        let txn = self.db.begin_read().context("failed to begin read")?;
        let table = txn.open_table(T::TABLE).context("failed to open table")?;
        let value = table.get(id).context("failed to get item from store")?;

        let Some(bytes) = value else {
            return Ok(None);
        };

        let item: T = if let Some(cipher) = &self.cipher {
            let decrypted =
                decrypt_bytes(cipher, bytes.value()).context("failed to decrypt data")?;
            postcard::from_bytes(&decrypted).context("failed to deserialize from postcard")?
        } else {
            postcard::from_bytes(bytes.value()).context("failed to deserialize from postcard")?
        };

        Ok(Some(item))
    }

    fn get_salt(&self) -> Result<String> {
        let value: Option<String> = self
            .get_meta(META_KEY_SALT)
            .context("failed to get salt from meta table")?;

        if let Some(salt) = value {
            Ok(salt)
        } else {
            let new_salt_string = SaltString::generate(&mut OsRng);
            self.set_meta(META_KEY_SALT, &new_salt_string.to_string())
                .context("failed to put salt in meta table")?;
            Ok(new_salt_string.to_string())
        }
    }

    fn get_meta<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: for<'a> Deserialize<'a>,
    {
        let txn = self.db.begin_read().context("failed to begin read")?;
        let table = txn.open_table(META_TABLE).context("failed to open table")?;
        let value = table.get(key).context("failed to get item from store")?;

        if let Some(bytes) = value {
            let item: T = postcard::from_bytes(bytes.value())
                .context("failed to deserialize from postcard")?;
            Ok(Some(item))
        } else {
            Ok(None)
        }
    }

    pub fn get_setting<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: for<'a> Deserialize<'a>,
    {
        let txn = self.db.begin_read().context("failed to begin read")?;
        let table = txn
            .open_table(SETTINGS_TABLE)
            .context("failed to open table")?;
        let value = table.get(key).context("failed to get item from store")?;

        let Some(bytes) = value else {
            return Ok(None);
        };

        let item: T = if let Some(cipher) = &self.cipher {
            let decrypted =
                decrypt_bytes(cipher, bytes.value()).context("failed to decrypt data")?;
            postcard::from_bytes(&decrypted).context("failed to deserialize from postcard")?
        } else {
            postcard::from_bytes(bytes.value()).context("failed to deserialize from postcard")?
        };

        Ok(Some(item))
    }

    fn set_meta<T>(&self, key: &str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        let txn = self.db.begin_write().context("failed to begin write")?;
        {
            let mut table = txn.open_table(META_TABLE).context("failed to open table")?;
            let bytes = postcard::to_stdvec(value).context("failed to serialize to postcard")?;
            table
                .insert(key, bytes.as_slice())
                .context("failed to insert into table")?;
        }
        txn.commit().context("failed to commit to database")?;
        Ok(())
    }

    pub fn set_setting<T>(&self, key: &str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        let txn = self.db.begin_write().context("failed to begin write")?;
        {
            let mut table = txn
                .open_table(SETTINGS_TABLE)
                .context("failed to open table")?;
            let bytes = postcard::to_stdvec(value).context("failed to serialize to postcard")?;

            let to_write: Vec<u8> = if let Some(cipher) = &self.cipher {
                encrypt_bytes(cipher, &bytes).context("failed to encrypt bytes")?
            } else {
                bytes
            };

            table
                .insert(key, to_write.as_slice())
                .context("failed to insert into table")?;
        }
        txn.commit().context("failed to commit to database")?;
        Ok(())
    }

    pub fn update<T>(&self, item: &T) -> Result<()>
    where
        T: TableModel,
    {
        if item.get_id().trim().is_empty() {
            return Err(anyhow!("id cannot be empty"));
        }

        let txn = self.db.begin_write().context("failed to begin write")?;
        {
            let mut table = txn.open_table(T::TABLE).context("failed to open table")?;
            let bytes = postcard::to_stdvec(&item).context("failed to serialize to postcard")?;

            let to_write: Vec<u8> = if let Some(cipher) = &self.cipher {
                encrypt_bytes(cipher, &bytes).context("failed to encrypt bytes")?
            } else {
                bytes
            };

            table
                .insert(item.get_id(), to_write.as_slice())
                .context("failed to update item")?;
        }
        txn.commit().context("failed to commit write to database")?;
        Ok(())
    }

    pub fn update_many<T>(&self, items: &[T]) -> Result<()>
    where
        T: TableModel,
    {
        let txn = self.db.begin_write().context("failed to begin write")?;
        {
            let mut table = txn.open_table(T::TABLE).context("failed to open table")?;
            for item in items {
                if item.get_id().trim().is_empty() {
                    return Err(anyhow!("id cannot be empty for update"));
                }
                let bytes = postcard::to_stdvec(item).context("failed to serialize to postcard")?;

                let to_write: Vec<u8> = if let Some(cipher) = &self.cipher {
                    encrypt_bytes(cipher, &bytes).context("failed to encrypt bytes")?
                } else {
                    bytes
                };

                table
                    .insert(item.get_id(), to_write.as_slice())
                    .context("failed to update item")?;
            }
        }
        txn.commit().context("failed to commit to database")?;
        Ok(())
    }

    pub fn export_table<T>(&self, pretty: bool) -> Result<String>
    where
        T: TableModel,
    {
        let all_items: Vec<T> = self.all().context("failed to get all items")?;
        let json = if pretty {
            serde_json::to_string_pretty(&all_items)
        } else {
            serde_json::to_string(&all_items)
        }
        .context("failed to serialize to JSON")?;
        Ok(json)
    }

    pub fn remove<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: TableModel,
    {
        let txn = self.db.begin_write().context("failed to begin write")?;
        let mut result = None;
        {
            let mut table = txn.open_table(T::TABLE).context("failed to open table")?;
            let maybe_bytes = table.remove(key).context("failed to remove item")?;

            if let Some(bytes) = maybe_bytes {
                let item: T = if let Some(cipher) = &self.cipher {
                    let decrypted =
                        decrypt_bytes(cipher, bytes.value()).context("failed to decrypt bytes")?;
                    postcard::from_bytes(&decrypted)
                        .context("failed to deserialize from postcard")?
                } else {
                    postcard::from_bytes(bytes.value())
                        .context("failed to deserialize from postcard")?
                };

                result = Some(item);
            }
        }
        txn.commit().context("failed to commit write")?;
        Ok(result)
    }

    pub fn remove_many<T>(&self, keys: &[&str]) -> Result<Vec<T>>
    where
        T: TableModel,
    {
        let txn = self.db.begin_write().context("failed to begin write")?;
        let mut result = Vec::new();
        {
            let mut table = txn.open_table(T::TABLE).context("failed to open table")?;
            for key in keys {
                let maybe_bytes = table.remove(key).context("failed to remove item")?;

                if let Some(bytes) = maybe_bytes {
                    let item: T = if let Some(cipher) = &self.cipher {
                        let decrypted = decrypt_bytes(cipher, bytes.value())
                            .context("failed to decrypt bytes")?;
                        postcard::from_bytes(&decrypted)
                            .context("failed to deserialize from postcard")?
                    } else {
                        postcard::from_bytes(bytes.value())
                            .context("failed to deserialize from postcard")?
                    };

                    result.push(item);
                }
            }
        }
        txn.commit().context("failed to commit write")?;
        Ok(result)
    }
}
