//! # Traits
//!
//! Some traits for minidb

use std::{
    fmt::{Debug, Display},
    fs::{create_dir_all, remove_file},
    path::Path,
};

use crate::{DBError, Database};
use anyhow::{Context, Result};
use cuid2::slug;
use minidb_utils::{deserialize_file, serialize_file};
use serde::{Deserialize, Serialize};

type ForeignKeyTuple<S> = (
    &'static str,
    &'static str,
    Box<dyn Fn(&S) -> Option<&str> + Send + Sync>,
);

/// A trait for defining a table
pub trait AsTable: Sized {
    /// The name of the table in `snake_case`
    fn name() -> &'static str;

    /// The primary key of the table
    fn get_id(&self) -> &Id<Self>;

    /// Sets the primary key of the table
    fn set_id(&mut self, id: Id<Self>);

    /// The foreign keys of the table
    fn get_foreign_keys() -> Vec<ForeignKeyTuple<Self>>;

    /// Inserts a record into the table and returns the ID
    ///
    /// ID will be generated automatically
    ///
    /// ## Arguments
    ///
    /// * `db` - The database instance
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
    fn insert(&self, db: &Database) -> Result<Id<Self>>
    where
        Self: Serialize,
    {
        let meta = db
            .metadata()
            .context(DBError::FailedToReadMetadata)?
            .context(DBError::NoMetadata)?;

        if meta.tables.is_empty() {
            return Err(DBError::NoTables.into());
        }

        let table_name = Self::name();
        if let Some(id) = &self.get_id().value {
            return Err(DBError::RecordAlreadyExists {
                table: table_name.to_string(),
                id: id.clone(),
            }
            .into());
        }

        for (field_name, ref_table, get_fk_id) in Self::get_foreign_keys() {
            let fk_id_option = get_fk_id(self);
            if let Some(fk_id_str) = fk_id_option {
                if !db.record_exists(ref_table, fk_id_str) {
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

        let path = db.path.read();
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

        let _lock = db.file_lock.write();
        serialize_file(&file_path, &self).context(DBError::FailedToSerializeFile(file_path))?;

        Ok(id)
    }

    /// Gets a record from a table
    ///
    /// ## Arguments
    ///
    /// * `db` - The database instance
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
    fn get(db: &Database, id: &Id<Self>) -> Result<Self>
    where
        Self: for<'de> Deserialize<'de>,
    {
        if id.is_none() {
            return Err(DBError::InvalidKey(id.to_string()).into());
        }

        let meta = db
            .metadata()
            .context(DBError::FailedToReadMetadata)?
            .context(DBError::NoMetadata)?;

        if meta.tables.is_empty() {
            return Err(DBError::NoTables.into());
        }

        let table_name = Self::name();
        let path = db.path.read();
        let table_dir_path = path.join(table_name);
        let file_path = table_dir_path.join(id);

        if !file_path.is_file() {
            return Err(DBError::RecordNotFound {
                table: table_name.to_string(),
                id: id.to_string(),
            }
            .into());
        }

        let _lock = db.file_lock.read();
        let mut record: Self =
            deserialize_file(&file_path).context(DBError::FailedToDeserializeFile(file_path))?;

        record.set_id(id.clone());

        Ok(record)
    }

    /// Deletes a record from a table
    ///
    /// ## Arguments
    ///
    /// * `db` - The database instance
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
    fn delete(db: &Database, id: &Id<Self>) -> Result<()> {
        if id.is_none() {
            return Err(DBError::InvalidKey(id.to_string()).into());
        }

        let meta = db
            .metadata()
            .context(DBError::FailedToReadMetadata)?
            .context(DBError::NoMetadata)?;

        if meta.tables.is_empty() {
            return Err(DBError::NoTables.into());
        }

        // TODO restrict deleting record if other tables have foreign keys pointing to it

        let table_name = Self::name();
        let path = db.path.read();
        let file_path = path.join(table_name).join(id);

        if !file_path.is_file() {
            return Err(DBError::RecordNotFound {
                table: table_name.to_string(),
                id: id.to_string(),
            }
            .into());
        }

        let _lock = db.file_lock.write();
        remove_file(&file_path).context(DBError::FailedToRemoveFile(file_path))?;

        Ok(())
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
    fn update(&self, db: &Database) -> Result<()>
    where
        Self: Serialize,
    {
        let id = self.get_id();

        if id.is_none() {
            return Err(DBError::InvalidKey(id.to_string()).into());
        }

        let meta = db
            .metadata()
            .context(DBError::FailedToReadMetadata)?
            .context(DBError::NoMetadata)?;

        if meta.tables.is_empty() {
            return Err(DBError::NoTables.into());
        }

        for (field_name, ref_table, get_fk_id) in Self::get_foreign_keys() {
            let fk_id_option = get_fk_id(self);
            if let Some(fk_id_str) = fk_id_option {
                if !db.record_exists(ref_table, fk_id_str) {
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

        let table_name = Self::name();
        let path = db.path.read();
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

        let _lock = db.file_lock.write();
        serialize_file(&file_path, &self).context(DBError::FailedToSerializeFile(file_path))
    }
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

impl<T> AsRef<Path> for Id<T> {
    fn as_ref(&self) -> &Path {
        match self.value {
            Some(ref s) => Path::new(s),
            None => Path::new(""),
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
