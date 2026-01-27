use anyhow::{Result, anyhow};
use minidb::{Store, TableModel};
use redb::TableDefinition;
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

struct CliDb {
    storage: Store,
}

impl std::ops::Deref for CliDb {
    type Target = Store;

    fn deref(&self) -> &Self::Target {
        &self.storage
    }
}

impl std::ops::DerefMut for CliDb {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.storage
    }
}

#[derive(Serialize, Deserialize)]
struct Restaurant {
    pub id: String,
}

impl TableModel for Restaurant {
    const TABLE: redb::TableDefinition<'_, &'static str, &[u8]> =
        TableDefinition::new("restaurants");

    fn get_id(&self) -> &str {
        &self.id
    }

    fn set_id(&mut self, id: String) {
        self.id = id;
    }
}

#[derive(Serialize, Deserialize)]
struct Order {
    pub id: String,
    pub restaurant_id: String,
}

impl TableModel for Order {
    const TABLE: TableDefinition<'_, &'static str, &[u8]> = TableDefinition::new("orders");

    fn get_id(&self) -> &str {
        &self.id
    }

    fn set_id(&mut self, id: String) {
        self.id = id;
    }
}

impl CliDb {
    pub fn place_order(&self, order: Order) -> Result<()> {
        if self.get::<Restaurant>(&order.restaurant_id)?.is_none() {
            return Err(anyhow!("Restaurant not found"));
        }

        self.insert(order)
    }

    pub fn all_restaurants(&self) -> Result<Vec<Restaurant>> {
        self.all()
    }
}

#[test]
fn test_store_place_order() {
    let temp_file = NamedTempFile::new().unwrap();
    let db = CliDb {
        storage: Store::new(temp_file).expect("failed to create storage"),
    };
    let r = Restaurant {
        id: "bca".to_string(),
    };
    let r2 = Restaurant {
        id: "bca2".to_string(),
    };
    db.insert(r).unwrap();
    db.insert(r2).unwrap();

    let o = Order {
        id: "abc".to_string(),
        restaurant_id: "bca".to_string(),
    };

    db.place_order(o).unwrap();

    let all = db.all_restaurants().unwrap();
    assert_eq!(all[0].id, "bca");
    assert_eq!(all[1].id, "bca2");

    let o2 = db.get::<Order>("abc").unwrap().unwrap();
    assert_eq!(o2.id, "abc");
    assert_eq!(o2.restaurant_id, "bca");
}
