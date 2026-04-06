// This Source Code Form is subject to the terms of the
// Mozilla Public License, v. 2.0. If a copy of the MPL was not distributed
// with this file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::{fmt::Debug, marker::PhantomData};

use crate::{
    encryption::decrypt_bytes,
    error::{Error, Result},
};
use chacha20poly1305::XChaCha20Poly1305;
use redb::{Range, TableDefinition};
use serde::{Deserialize, Serialize};

/// A table model. A table model is a struct that implements the [`Table`] trait.
///
/// ## Example
///
/// ```rust,no_run
/// use minidb::{
///     serde::{Deserialize, Serialize},
///     Table,
/// };
///
/// #[derive(Serialize, Deserialize)]
/// #[serde(crate = "minidb::serde")] // required if using re-exported serde
/// struct Person {
///     id: String,
///     name: String,
///     age: u8,
/// }
///
/// impl Table for Person {
///     const TABLE: redb::TableDefinition<'_, &'static str, &[u8]> = redb::TableDefinition::new("people");
///
///     fn get_id(&self) -> &str {
///         &self.id
///     }
///
///     fn set_id(&mut self, id: String) {
///         self.id = id;
///     }
/// }
/// ```
pub trait Table: Serialize + for<'de> Deserialize<'de> {
    /// The table definition
    const TABLE: TableDefinition<'_, &'static str, &[u8]>;

    /// Returns the id of the table model
    fn get_id(&self) -> &str;

    /// Sets the id of the table model
    fn set_id(&mut self, id: String);
}

/// An iterator over a table's items, with optional decryption
pub struct TableIterator<'a, T> {
    inner: Range<'a, &'static str, &'static [u8]>,
    cipher: Option<&'a XChaCha20Poly1305>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> Debug for TableIterator<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TableIterator").finish_non_exhaustive()
    }
}

impl<'a, T> TableIterator<'a, T> {
    /// Creates a new [`TableIterator`]
    ///
    /// ## Arguments
    ///
    /// * `inner` - The range of items from the redb table
    ///
    /// ## Returns
    ///
    /// A new [`TableIterator`]
    #[must_use]
    pub fn new(inner: Range<'a, &'static str, &'static [u8]>) -> Self {
        Self {
            inner,
            cipher: None,
            _phantom: PhantomData,
        }
    }

    /// Adds a cipher to the iterator for decryption
    ///
    /// ## Arguments
    ///
    /// * `cipher` - The cipher to use for decryption
    ///
    /// ## Returns
    ///
    /// The [`TableIterator`] with the cipher added
    #[must_use]
    pub fn with_cipher(mut self, cipher: &'a XChaCha20Poly1305) -> Self {
        self.cipher = Some(cipher);
        self
    }
}

impl<T> Iterator for TableIterator<'_, T>
where
    T: for<'de> Deserialize<'de>,
{
    type Item = Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.inner.next()?;

        match result {
            Ok((_key, value)) => {
                let bytes = if let Some(cipher) = &self.cipher {
                    match decrypt_bytes(cipher, value.value()) {
                        Ok(d) => d,
                        Err(e) => return Some(Err(e)),
                    }
                } else {
                    value.value().to_vec()
                };

                match postcard::from_bytes(&bytes) {
                    Ok(item) => Some(Ok(item)),
                    Err(e) => Some(Err(Error::Serialization(e))),
                }
            }
            Err(e) => Some(Err(Error::Storage(e))),
        }
    }
}
