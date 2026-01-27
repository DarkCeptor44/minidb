pub use redb;

use std::path::Path;

use anyhow::{Context, Result};
use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};

const SETTINGS_TABLE: TableDefinition<&'static str, &[u8]> = TableDefinition::new("settings");

pub trait TableModel: Serialize + for<'de> Deserialize<'de> {
    const TABLE: TableDefinition<'_, &'static str, &[u8]>;

    fn get_id(&self) -> &str;
    fn set_id(&mut self, id: String);
}

pub struct Store {
    db: Database,
}

impl Store {
    pub fn new<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let db = Database::builder()
            .create(path)
            .context("failed to create database")?;
        let store = Self { db };
        store
            .create_table_impl(SETTINGS_TABLE)
            .context("failed to bootstrap tables")?;

        Ok(store)
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
            let decoded: T = postcard::from_bytes(value.value())
                .context("failed to deserialize from postcard")?;
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
            let data: T = postcard::from_bytes(value.value())?;
            f(data);
        }

        Ok(())
    }

    pub fn insert<T>(&self, mut item: T) -> Result<()>
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
            let bytes = postcard::to_stdvec(&item).context("failed to serialize to postcard")?;
            table
                .insert(item.get_id(), bytes.as_slice())
                .context("failed to insert into table")?;
        }
        txn.commit().context("failed to commit to database")?;
        Ok(())
    }

    pub fn get<T>(&self, id: &str) -> Result<Option<T>>
    where
        T: TableModel,
    {
        let txn = self.db.begin_read().context("failed to begin read")?;
        let table = txn.open_table(T::TABLE).context("failed to open table")?;
        let value = table.get(id).context("failed to get item from store")?;

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

        if let Some(bytes) = value {
            let item: T = postcard::from_bytes(bytes.value())
                .context("failed to deserialize from postcard")?;
            Ok(Some(item))
        } else {
            Ok(None)
        }
    }

    pub fn remove<T>(&self, key: &str) -> Result<()>
    where
        T: TableModel,
    {
        let txn = self.db.begin_write().context("failed to begin write")?;
        {
            let mut table = txn.open_table(T::TABLE).context("failed to open table")?;
            table.remove(key).context("failed to remove item")?;
        }
        txn.commit().context("failed to commit write")?;
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
            table
                .insert(key, bytes.as_slice())
                .context("failed to insert into table")?;
        }
        txn.commit().context("failed to commit to database")?;
        Ok(())
    }
}
