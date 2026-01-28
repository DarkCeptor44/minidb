pub use redb;

use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};
use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition, WriteTransaction};
use serde::{Deserialize, Serialize};

const SETTINGS_TABLE: TableDefinition<&'static str, &[u8]> = TableDefinition::new("settings");

type Initializer = Box<dyn Fn(&WriteTransaction) -> Result<()>>;

pub trait TableModel: Serialize + for<'de> Deserialize<'de> {
    const TABLE: TableDefinition<'_, &'static str, &[u8]>;

    fn get_id(&self) -> &str;
    fn set_id(&mut self, id: String);
}

pub struct StoreBuilder {
    path: PathBuf,
    initializers: Vec<Initializer>,
}

impl StoreBuilder {
    pub fn new<P>(path: P) -> Self
    where
        P: Into<PathBuf>,
    {
        Self {
            path: path.into(),
            initializers: Vec::new(),
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

    pub fn build(self) -> Result<Store> {
        let db = Database::builder()
            .create(&self.path)
            .context("failed to create database file")?;

        let txn = db.begin_write().context("failed to begin bootstrap txn")?;
        for init in self.initializers {
            init(&txn)?;
        }
        txn.commit().context("failed to commit bootstrap")?;

        Ok(Store { db })
    }
}

pub struct Store {
    db: Database,
}

impl Store {
    pub fn builder<P>(path: P) -> StoreBuilder
    where
        P: Into<PathBuf>,
    {
        StoreBuilder::new(path)
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
                table
                    .insert(item.get_id(), bytes.as_slice())
                    .context("failed to insert into table")?;
            }
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

    pub fn update<T>(&self, item: T) -> Result<()>
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
            table
                .insert(item.get_id(), bytes.as_slice())
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
                table
                    .insert(item.get_id(), bytes.as_slice())
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
}
