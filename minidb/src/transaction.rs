// This Source Code Form is subject to the terms of the
// Mozilla Public License, v. 2.0. If a copy of the MPL was not distributed
// with this file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{
    SETTINGS_TABLE,
    encryption::{decrypt_bytes, encrypt_bytes},
    error::{Error, Result},
    ipc::{IpcClient, IpcRequest, IpcResponse},
    model::Table,
};
use chacha20poly1305::XChaCha20Poly1305;
use serde::Serialize;
use std::fmt::Debug;

pub(crate) enum TransactionBackend<'a> {
    Local(Box<redb::WriteTransaction>),
    Ipc(&'a IpcClient),
}

impl Debug for TransactionBackend<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionBackend::Local(_) => f
                .debug_tuple("Local")
                .field(&"<redb::WriteTransaction>")
                .finish(),
            TransactionBackend::Ipc(client) => f.debug_tuple("Ipc").field(client).finish(),
        }
    }
}

/// A write transaction.
///
/// This struct allows grouping multiple database operations within a single, atomic transaction.
/// It is created by calling [`MiniDB::transaction`](crate::MiniDB::transaction).
pub struct Transaction<'a> {
    pub(crate) backend: TransactionBackend<'a>,
    pub(crate) cipher: Option<&'a XChaCha20Poly1305>,
}

impl Debug for Transaction<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Transaction").finish_non_exhaustive()
    }
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
        T: Table,
    {
        if item.get_id().trim().is_empty() {
            let id = cuid2::slug();
            item.set_id(id);
        }

        let bytes = postcard::to_stdvec(item)?;
        let to_write: Vec<u8> = if let Some(cipher) = self.cipher {
            encrypt_bytes(cipher, &bytes)?
        } else {
            bytes
        };

        match &self.backend {
            TransactionBackend::Local(txn) => {
                let mut table = txn.open_table(T::TABLE)?;
                table.insert(item.get_id(), to_write.as_slice())?;
            }
            TransactionBackend::Ipc(client) => {
                match client.send_request(&IpcRequest::Insert {
                    table: T::TABLE.to_string(),
                    key: item.get_id().to_string(),
                    value: to_write,
                })? {
                    IpcResponse::Ok => {}
                    IpcResponse::Error(e) => return Err(Error::Ipc(e)),
                    _ => return Err(Error::UnexpectedIpcResponse),
                }
            }
        }

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
        T: Table,
    {
        let mut new_items = Vec::new();
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

            new_items.push((item.get_id().to_string(), to_write));
        }

        match &self.backend {
            TransactionBackend::Local(txn) => {
                let mut table = txn.open_table(T::TABLE)?;
                for (id, to_write) in new_items {
                    table.insert(&*id, to_write.as_slice())?;
                }
            }
            TransactionBackend::Ipc(client) => {
                match client.send_request(&IpcRequest::InsertMany {
                    table: T::TABLE.to_string(),
                    items: new_items,
                })? {
                    IpcResponse::Ok => {}
                    IpcResponse::Error(e) => return Err(Error::Ipc(e)),
                    _ => return Err(Error::UnexpectedIpcResponse),
                }
            }
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
        T: Table,
    {
        if item.get_id().trim().is_empty() {
            return Err(Error::EmptyID);
        }

        let bytes = postcard::to_stdvec(item)?;
        let to_write: Vec<u8> = if let Some(cipher) = &self.cipher {
            encrypt_bytes(cipher, &bytes)?
        } else {
            bytes
        };

        match &self.backend {
            TransactionBackend::Local(txn) => {
                let mut table = txn.open_table(T::TABLE)?;
                table.insert(item.get_id(), to_write.as_slice())?;
            }
            TransactionBackend::Ipc(client) => {
                match client.send_request(&IpcRequest::Update {
                    table: T::TABLE.to_string(),
                    key: item.get_id().to_string(),
                    value: to_write,
                })? {
                    IpcResponse::Ok => {}
                    IpcResponse::Error(e) => return Err(Error::Ipc(e)),
                    _ => return Err(Error::UnexpectedIpcResponse),
                }
            }
        }

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
        T: Table,
    {
        let mut new_items = Vec::new();
        for item in items {
            if item.get_id().trim().is_empty() {
                return Err(Error::EmptyID);
            }

            let bytes = postcard::to_stdvec(&item)?;
            let to_write = if let Some(cipher) = &self.cipher {
                encrypt_bytes(cipher, &bytes)?
            } else {
                bytes
            };

            new_items.push((item.get_id().to_string(), to_write));
        }

        match &self.backend {
            TransactionBackend::Local(txn) => {
                let mut table = txn.open_table(T::TABLE)?;
                for (id, to_write) in new_items {
                    table.insert(&*id, to_write.as_slice())?;
                }
            }
            TransactionBackend::Ipc(client) => {
                match client.send_request(&IpcRequest::UpdateMany {
                    table: T::TABLE.to_string(),
                    items: new_items,
                })? {
                    IpcResponse::Ok => {}
                    IpcResponse::Error(e) => return Err(Error::Ipc(e)),
                    _ => return Err(Error::UnexpectedIpcResponse),
                }
            }
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
        T: Table,
    {
        let maybe_bytes = match &self.backend {
            TransactionBackend::Local(txn) => {
                let mut table = txn.open_table(T::TABLE)?;
                table.remove(key)?.map(|v| v.value().to_vec())
            }
            TransactionBackend::Ipc(client) => {
                match client.send_request(&IpcRequest::Remove {
                    table: T::TABLE.to_string(),
                    key: key.to_string(),
                })? {
                    IpcResponse::Value(bytes) => bytes,
                    IpcResponse::Error(e) => return Err(Error::Ipc(e)),
                    _ => return Err(Error::UnexpectedIpcResponse),
                }
            }
        };

        if let Some(bytes) = maybe_bytes {
            let item: T = if let Some(cipher) = &self.cipher {
                let decrypted = decrypt_bytes(cipher, &bytes)?;
                postcard::from_bytes(&decrypted)?
            } else {
                postcard::from_bytes(&bytes)?
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
        T: Table,
    {
        let values: Vec<Vec<u8>> = match &self.backend {
            TransactionBackend::Local(txn) => {
                let mut results = Vec::new();
                let mut table = txn.open_table(T::TABLE)?;
                for key in keys {
                    if let Some(bytes) = table.remove(key)? {
                        results.push(bytes.value().to_vec());
                    }
                }
                results
            }
            TransactionBackend::Ipc(client) => {
                match client.send_request(&IpcRequest::RemoveMany {
                    table: T::TABLE.to_string(),
                    keys: keys.iter().map(std::string::ToString::to_string).collect(),
                })? {
                    IpcResponse::Rows(items) => items.into_iter().map(|(_, v)| v).collect(),
                    IpcResponse::Error(e) => return Err(Error::Ipc(e)),
                    _ => return Err(Error::UnexpectedIpcResponse),
                }
            }
        };

        let mut decrypted_items = Vec::with_capacity(values.len());
        for bytes in values {
            let data: T = if let Some(cipher) = &self.cipher {
                let decrypted = decrypt_bytes(cipher, &bytes)?;
                postcard::from_bytes(&decrypted)?
            } else {
                postcard::from_bytes(&bytes)?
            };

            decrypted_items.push(data);
        }

        Ok(decrypted_items)
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
        let bytes = postcard::to_stdvec(value)?;
        let to_write: Vec<u8> = if let Some(cipher) = &self.cipher {
            encrypt_bytes(cipher, &bytes)?
        } else {
            bytes
        };

        match &self.backend {
            TransactionBackend::Local(txn) => {
                let mut table = txn.open_table(SETTINGS_TABLE)?;
                table.insert(key, to_write.as_slice())?;
            }
            TransactionBackend::Ipc(client) => {
                match client.send_request(&IpcRequest::SetSetting {
                    key: key.to_string(),
                    value: to_write,
                })? {
                    IpcResponse::Ok => {}
                    IpcResponse::Error(e) => return Err(Error::Ipc(e)),
                    _ => return Err(Error::UnexpectedIpcResponse),
                }
            }
        }

        Ok(())
    }
}
