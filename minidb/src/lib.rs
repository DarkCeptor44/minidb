pub mod types;

use std::{path::Path, sync::Arc};

use anyhow::{Context, Result};
use redb::{Database, ReadableDatabase, TableDefinition};
use serde::{Deserialize, Serialize};
use types::Id;

pub trait TableModel: Serialize + for<'de> Deserialize<'de> {
    const TABLE: TableDefinition<'_, Id, &[u8]>;

    fn get_key(&self) -> Id;
    fn set_id(&mut self, id: Id);
}

pub struct Store {
    db: Arc<Database>,
}

impl Store {
    pub fn new<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let db = Database::builder()
            .create(path)
            .context("failed to create database")?;
        Ok(Self { db: Arc::new(db) })
    }

    pub fn insert<T>(&self, item: &mut T) -> Result<()>
    where
        T: TableModel,
    {
        if item.get_key().is_none() {
            let id = Id::generate();
            item.set_id(id);
        }

        let txn = self.db.begin_write().context("failed to begin write")?;
        {
            let mut table = txn.open_table(T::TABLE).context("failed to open table")?;
            let bytes = postcard::to_stdvec(item).context("failed to serialize to postcard")?;
            table
                .insert(item.get_key(), bytes.as_slice())
                .context("failed to insert into table")?;
        }
        txn.commit().context("failed to commit to database")?;
        Ok(())
    }

    pub fn get<T>(&self, id: &Id) -> Result<Option<T>>
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
}
