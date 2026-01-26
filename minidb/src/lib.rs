use std::path::Path;

use anyhow::{Context, Result};
use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};

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
        Ok(Self { db })
    }

    pub fn save<T>(&self, mut item: T) -> Result<()>
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
}
