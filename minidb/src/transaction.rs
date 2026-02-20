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

use crate::SETTINGS_TABLE;
use crate::encryption::{decrypt_bytes, encrypt_bytes};
use crate::model::TableModel;
use anyhow::{Context, Result, anyhow};
use chacha20poly1305::XChaCha20Poly1305;
use redb::WriteTransaction;
use serde::Serialize;

/// A write transaction.
///
/// This struct allows grouping multiple database operations within a single, atomic transaction.
/// It is created by calling [`MiniDB::transaction`](crate::MiniDB::transaction).
pub struct Transaction<'a> {
    pub(crate) txn: WriteTransaction,
    pub(crate) cipher: Option<&'a XChaCha20Poly1305>,
}

impl Transaction<'_> {
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
    /// db.transaction(|txn| {
    ///     txn.insert(&mut person)?;
    ///     Ok(())
    /// }).unwrap();
    /// ```
    pub fn insert<T>(&self, item: &mut T) -> Result<()>
    where
        T: TableModel,
    {
        if item.get_id().trim().is_empty() {
            let id = cuid2::slug();
            item.set_id(id);
        }

        let mut table = self
            .txn
            .open_table(T::TABLE)
            .context("failed to open table")?;
        let bytes = postcard::to_stdvec(item).context("failed to serialize to postcard")?;

        let to_write: Vec<u8> = if let Some(cipher) = self.cipher {
            encrypt_bytes(cipher, &bytes).context("failed to encrypt bytes")?
        } else {
            bytes
        };

        table
            .insert(item.get_id(), to_write.as_slice())
            .context("failed to insert into table")?;
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
    /// db.transaction(|txn| {
    ///     txn.insert_many(&mut people)?;
    ///     Ok(())
    /// }).unwrap();
    /// ```
    pub fn insert_many<T>(&self, items: &mut [T]) -> Result<()>
    where
        T: TableModel,
    {
        let mut table = self
            .txn
            .open_table(T::TABLE)
            .context("failed to open table")?;
        for item in items {
            if item.get_id().trim().is_empty() {
                let id = cuid2::slug();
                item.set_id(id);
            }

            let bytes = postcard::to_stdvec(&item).context("failed to serialize to postcard")?;

            let to_write: Vec<u8> = if let Some(cipher) = self.cipher {
                encrypt_bytes(cipher, &bytes).context("failed to encrypt bytes")?
            } else {
                bytes
            };

            table
                .insert(item.get_id(), to_write.as_slice())
                .context("failed to insert into table")?;
        }
        Ok(())
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
    /// db.transaction(|txn| {
    ///     txn.update(&person)?;
    ///     Ok(())
    /// }).unwrap();
    /// ```
    pub fn update<T>(&self, item: &T) -> Result<()>
    where
        T: TableModel,
    {
        if item.get_id().trim().is_empty() {
            return Err(anyhow!("id cannot be empty"));
        }

        let mut table = self
            .txn
            .open_table(T::TABLE)
            .context("failed to open table")?;
        let bytes = postcard::to_stdvec(&item).context("failed to serialize to postcard")?;

        let to_write: Vec<u8> = if let Some(cipher) = self.cipher {
            encrypt_bytes(cipher, &bytes).context("failed to encrypt bytes")?
        } else {
            bytes
        };

        table
            .insert(item.get_id(), to_write.as_slice())
            .context("failed to update item")?;
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
    /// db.transaction(|txn| {
    ///     txn.update_many(&people)?;
    ///     Ok(())
    /// }).unwrap();
    /// ```
    pub fn update_many<T>(&self, items: &[T]) -> Result<()>
    where
        T: TableModel,
    {
        let mut table = self
            .txn
            .open_table(T::TABLE)
            .context("failed to open table")?;
        for item in items {
            if item.get_id().trim().is_empty() {
                return Err(anyhow!("id cannot be empty for update"));
            }
            let bytes = postcard::to_stdvec(item).context("failed to serialize to postcard")?;

            let to_write: Vec<u8> = if let Some(cipher) = self.cipher {
                encrypt_bytes(cipher, &bytes).context("failed to encrypt bytes")?
            } else {
                bytes
            };

            table
                .insert(item.get_id(), to_write.as_slice())
                .context("failed to update item")?;
        }
        Ok(())
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
    /// Returns an error if the table is not found, if the table is not initialized, or if the decryption/deserialization fails
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// db.transaction(|txn| {
    ///     let removed = txn.remove::<Person>("id")?;
    ///     Ok(())
    /// }).unwrap();
    /// ```
    pub fn remove<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: TableModel,
    {
        let mut table = self
            .txn
            .open_table(T::TABLE)
            .context("failed to open table")?;
        let maybe_bytes = table.remove(key).context("failed to remove item")?;

        if let Some(bytes) = maybe_bytes {
            let item: T = if let Some(cipher) = self.cipher {
                let decrypted =
                    decrypt_bytes(cipher, bytes.value()).context("failed to decrypt bytes")?;
                postcard::from_bytes(&decrypted).context("failed to deserialize from postcard")?
            } else {
                postcard::from_bytes(bytes.value())
                    .context("failed to deserialize from postcard")?
            };

            Ok(Some(item))
        } else {
            Ok(None)
        }
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
    /// Returns an error if the table is not found, if the table is not initialized, or if the decryption/deserialization fails
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// db.transaction(|txn| {
    ///     let removed = txn.remove_many::<Person>(&["id1", "id2"])?;
    ///     Ok(())
    /// }).unwrap();
    /// ```
    pub fn remove_many<T>(&self, keys: &[&str]) -> Result<Vec<T>>
    where
        T: TableModel,
    {
        let mut result = Vec::new();
        let mut table = self
            .txn
            .open_table(T::TABLE)
            .context("failed to open table")?;
        for key in keys {
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

                result.push(item);
            }
        }
        Ok(result)
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
    /// db.transaction(|txn| {
    ///     txn.set_setting("key", &"value".to_string())?;
    ///     Ok(())
    /// }).unwrap();
    /// ```
    pub fn set_setting<T>(&self, key: &str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        let mut table = self
            .txn
            .open_table(SETTINGS_TABLE)
            .context("failed to open table")?;
        let bytes = postcard::to_stdvec(value).context("failed to serialize to postcard")?;

        let to_write: Vec<u8> = if let Some(cipher) = self.cipher {
            encrypt_bytes(cipher, &bytes).context("failed to encrypt bytes")?
        } else {
            bytes
        };

        table
            .insert(key, to_write.as_slice())
            .context("failed to insert into table")?;
        Ok(())
    }
}
