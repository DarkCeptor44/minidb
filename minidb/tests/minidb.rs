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

use std::collections::HashSet;

use anyhow::{Result, anyhow};
use minidb::{MiniDB, TableModel};
use rand::seq::IndexedRandom;
use redb::TableDefinition;
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

struct CliDb {
    storage: MiniDB,
}

impl std::ops::Deref for CliDb {
    type Target = MiniDB;

    fn deref(&self) -> &Self::Target {
        &self.storage
    }
}

impl std::ops::DerefMut for CliDb {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.storage
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
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
    pub fn place_order(&self, order: &mut Order) -> Result<()> {
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
fn test_minidb_insert() {
    let temp_file = NamedTempFile::new().expect("failed to create temp file");
    let db = MiniDB::builder(temp_file.path())
        .table::<Restaurant>()
        .table::<Order>()
        .build()
        .expect("failed to build store");

    let mut r = Restaurant { id: String::new() };
    db.insert(&mut r).expect("failed to insert restaurant");

    let mut o = Order {
        id: String::new(),
        restaurant_id: r.id,
    };
    db.insert(&mut o).expect("failed to insert order");

    let all_orders = db.all::<Order>().expect("failed to get all orders");
    assert_eq!(all_orders.len(), 1);

    let all_restaurants = db
        .all::<Restaurant>()
        .expect("failed to get all restaurants");
    assert_eq!(all_restaurants.len(), 1);
}

#[test]
fn test_minidb_insert_many() {
    const N: usize = 1000;

    let temp_file = NamedTempFile::new().expect("failed to create temp file");
    let db = MiniDB::builder(temp_file.path())
        .table::<Restaurant>()
        .table::<Order>()
        .build()
        .expect("failed to build store");

    let mut restaurants: Vec<Restaurant> =
        (0..N).map(|_| Restaurant { id: String::new() }).collect();

    db.insert_many(&mut restaurants)
        .expect("failed to insert many restaurants");

    let all_restaurants = db
        .all::<Restaurant>()
        .expect("failed to get all restaurants");
    assert_eq!(all_restaurants.len(), N);

    let mut ids = HashSet::new();
    for r in all_restaurants {
        ids.insert(r.id);
    }
    assert_eq!(ids.len(), N);
}

#[test]
fn test_minidb_update() {
    let temp_file = NamedTempFile::new().expect("failed to create temp file");
    let db = MiniDB::builder(temp_file.path())
        .table::<Restaurant>()
        .table::<Order>()
        .build()
        .expect("failed to build store");

    let mut r1 = Restaurant { id: String::new() };
    db.insert(&mut r1).expect("failed to insert restaurant");

    let mut r2 = Restaurant { id: String::new() };
    db.insert(&mut r2).expect("failed to insert restaurant");

    let mut o = Order {
        id: String::new(),
        restaurant_id: r1.id,
    };
    db.insert(&mut o).expect("failed to insert order");

    let mut o: Order = db
        .get(&o.id)
        .expect("failed to get order")
        .expect("order is not in store for some reason");
    o.restaurant_id = r2.id;
    db.update(&o).expect("failed to update order");

    let all_orders = db.all::<Order>().expect("failed to get orders");
    let order = all_orders.first().expect("orders is empty for some reason");

    assert_eq!(order.id, o.id);
    assert_eq!(order.restaurant_id, o.restaurant_id);
}

#[test]
fn test_minidb_update_many() {
    const N: usize = 1000;

    let temp_file = NamedTempFile::new().expect("failed to create temp file");
    let db = MiniDB::builder(temp_file.path())
        .table::<Restaurant>()
        .table::<Order>()
        .build()
        .expect("failed to build store");

    let mut r1 = Restaurant { id: String::new() };
    db.insert(&mut r1).expect("failed to insert restaurant");

    let mut r2 = Restaurant { id: String::new() };
    db.insert(&mut r2).expect("failed to insert restaurant");

    let mut orders: Vec<Order> = (0..N)
        .map(|_| Order {
            id: String::new(),
            restaurant_id: r1.id.clone(),
        })
        .collect();

    db.insert_many(&mut orders)
        .expect("failed to insert many orders");

    orders
        .iter_mut()
        .for_each(|o| o.restaurant_id = r2.id.clone());
    db.update_many(&orders)
        .expect("failed to update many orders");

    let all_orders = db.all::<Order>().expect("failed to get all orders");
    assert_eq!(all_orders.len(), N);
    let mut ids = HashSet::new();
    for o in all_orders {
        ids.insert(o.id);
        assert_eq!(o.restaurant_id, r2.id);
    }
    assert_eq!(ids.len(), N);
}

#[test]
fn test_minidb_get() {
    const N: usize = 1000;

    let temp_file = NamedTempFile::new().expect("failed to create temp file");
    let db = MiniDB::builder(temp_file.path())
        .table::<Restaurant>()
        .table::<Order>()
        .build()
        .expect("failed to build store");

    let mut restaurants = Vec::new();
    for _ in 0..N {
        restaurants.push(Restaurant { id: String::new() });
    }
    db.insert_many(&mut restaurants)
        .expect("failed to insert many restaurants");

    let mut rng = rand::rng();
    let random_restaurant = restaurants
        .choose(&mut rng)
        .expect("failed to choose a random restaurant");
    let r: Restaurant = db
        .get(&random_restaurant.id)
        .expect("failed to get restaurant")
        .expect("restaurants is empty for some reason");
    assert_eq!(r.id, random_restaurant.id);
}

#[test]
#[should_panic(expected = "restaurant is non-existent for some reason")]
fn test_minidb_get_non_existent() {
    const N: usize = 1000;

    let temp_file = NamedTempFile::new().expect("failed to create temp file");
    let db = MiniDB::builder(temp_file.path())
        .table::<Restaurant>()
        .table::<Order>()
        .build()
        .expect("failed to build store");

    let mut restaurants = Vec::new();
    for _ in 0..N {
        restaurants.push(Restaurant { id: String::new() });
    }
    db.insert_many(&mut restaurants)
        .expect("failed to insert many restaurants");

    db.get::<Restaurant>("askdjkasjk")
        .expect("failed to get restaurant")
        .expect("restaurant is non-existent for some reason");
}

#[test]
fn test_minidb_all() {
    const N: usize = 1000;

    let temp_file = NamedTempFile::new().expect("failed to create temp file");
    let db = MiniDB::builder(temp_file.path())
        .table::<Restaurant>()
        .table::<Order>()
        .build()
        .expect("failed to build store");

    let mut restaurants = Vec::new();
    for _ in 0..N {
        restaurants.push(Restaurant { id: String::new() });
    }
    db.insert_many(&mut restaurants)
        .expect("failed to insert many restaurants");

    let all_restaurants = db
        .all::<Restaurant>()
        .expect("failed to get all restaurants");
    assert_eq!(all_restaurants.len(), N);
}

#[test]
fn test_minidb_all_from_empty_table() {
    let temp_file = NamedTempFile::new().expect("failed to create temp file");
    let db = MiniDB::builder(temp_file.path())
        .table::<Restaurant>()
        .table::<Order>()
        .build()
        .expect("failed to build store");

    let all_restaurants = db
        .all::<Restaurant>()
        .expect("failed to get all restaurants");
    assert_eq!(all_restaurants.len(), 0);
}

#[test]
fn test_minidb_remove() {
    const N: usize = 1000;

    let temp_file = NamedTempFile::new().expect("failed to create temp file");
    let db = MiniDB::builder(temp_file.path())
        .table::<Restaurant>()
        .table::<Order>()
        .build()
        .expect("failed to build store");

    let mut restaurants = Vec::new();
    for _ in 0..N {
        restaurants.push(Restaurant { id: String::new() });
    }
    db.insert_many(&mut restaurants)
        .expect("failed to insert many restaurants");

    let mut rng = rand::rng();
    let random_rest = restaurants
        .choose(&mut rng)
        .expect("failed to choose a restaurant");

    db.remove::<Restaurant>(&random_rest.id)
        .expect("failed to remove restaurant")
        .expect("restaurant should exist but doesnt");

    assert!(
        db.get::<Restaurant>(&random_rest.id)
            .expect("failed to get restaurant")
            .is_none()
    );
}

#[test]
fn test_minidb_remove_many() {
    const N: usize = 1000;

    let temp_file = NamedTempFile::new().expect("failed to create temp file");
    let db = MiniDB::builder(temp_file.path())
        .table::<Restaurant>()
        .table::<Order>()
        .build()
        .expect("failed to build store");

    let mut restaurants = Vec::new();
    for _ in 0..N {
        restaurants.push(Restaurant { id: String::new() });
    }
    db.insert_many(&mut restaurants)
        .expect("failed to insert many restaurants");

    let restaurants_ids: Vec<&str> = restaurants.iter().map(|r| r.id.as_str()).collect();
    db.remove_many::<Restaurant>(&restaurants_ids)
        .expect("failed to remove restaurant");

    assert_eq!(
        db.all::<Restaurant>()
            .expect("failed to get all restaurants")
            .len(),
        0
    );
}

#[test]
fn test_minidb_for_each() {
    const N: usize = 1000;

    let temp_file = NamedTempFile::new().expect("failed to create temp file");
    let db = MiniDB::builder(temp_file.path())
        .table::<Restaurant>()
        .table::<Order>()
        .build()
        .expect("failed to build store");

    let mut restaurants = Vec::new();
    for _ in 0..N {
        restaurants.push(Restaurant { id: String::new() });
    }
    db.insert_many(&mut restaurants)
        .expect("failed to insert many restaurants");

    let mut count = 0;
    db.for_each(|_r: &Restaurant| count += 1)
        .expect("failed to run for each");
    assert_eq!(count, N);
}

#[test]
fn test_minidb_settings() {
    let temp_file = NamedTempFile::new().expect("failed to create temp file");
    let db = MiniDB::builder(temp_file.path())
        .table::<Restaurant>()
        .table::<Order>()
        .build()
        .expect("failed to build store");

    assert!(
        db.get_setting::<Restaurant>("favorite_restaurant")
            .expect("failed to get setting")
            .is_none()
    );

    let rest = Restaurant {
        id: String::from("asjdasjksj"),
    };
    db.set_setting("favorite_restaurant", &rest)
        .expect("failed to set setting");

    let r: Restaurant = db
        .get_setting("favorite_restaurant")
        .expect("failed to get setting")
        .expect("restaurant should exist but doesnt");
    assert_eq!(r.id, rest.id);
}

#[test]
fn test_minidb_export_table() {
    const N: usize = 1000;

    let temp_file = NamedTempFile::new().expect("failed to create temp file");
    let db = MiniDB::builder(temp_file.path())
        .table::<Restaurant>()
        .table::<Order>()
        .build()
        .expect("failed to build store");

    let mut restaurants = Vec::new();
    for _ in 0..N {
        restaurants.push(Restaurant { id: String::new() });
    }
    db.insert_many(&mut restaurants)
        .expect("failed to insert many restaurants");

    restaurants.sort_unstable();

    let exported = db
        .export_table::<Restaurant>(false)
        .expect("failed to export restaurants");
    let json = serde_json::to_string(&restaurants).expect("failed to serialize to json");

    assert_eq!(exported, json);
}

#[test]
fn test_minidb_place_order() {
    let temp_file = NamedTempFile::new().unwrap();
    let db = CliDb {
        storage: MiniDB::builder(temp_file.path())
            .table::<Restaurant>()
            .table::<Order>()
            .build()
            .expect("failed to create storage"),
    };
    let mut r = Restaurant {
        id: "bca".to_string(),
    };
    let mut r2 = Restaurant {
        id: "bca2".to_string(),
    };
    db.insert(&mut r).unwrap();
    db.insert(&mut r2).unwrap();

    let mut o = Order {
        id: "abc".to_string(),
        restaurant_id: "bca".to_string(),
    };

    db.place_order(&mut o).unwrap();

    let all = db.all_restaurants().unwrap();
    assert_eq!(all[0].id, "bca");
    assert_eq!(all[1].id, "bca2");

    let o2 = db.get::<Order>("abc").unwrap().unwrap();
    assert_eq!(o2.id, "abc");
    assert_eq!(o2.restaurant_id, "bca");
}

#[test]
fn test_minidb_is_empty() {
    let temp_file = NamedTempFile::new().unwrap();
    let db = CliDb {
        storage: MiniDB::builder(temp_file.path())
            .table::<Restaurant>()
            .build()
            .expect("failed to create storage"),
    };

    assert!(db.is_empty::<Restaurant>().unwrap());
}

#[test]
fn test_minidb_transaction() {
    let temp_file = NamedTempFile::new().expect("failed to create temp file");
    let db = MiniDB::builder(temp_file.path())
        .table::<Restaurant>()
        .table::<Order>()
        .build()
        .expect("failed to build store");

    let mut r = Restaurant {
        id: "rest1".to_string(),
    };
    let mut o = Order {
        id: "order1".to_string(),
        restaurant_id: "rest1".to_string(),
    };

    db.transaction(|txn| {
        txn.insert(&mut r)?;
        txn.insert(&mut o)?;
        Ok(())
    })
    .expect("transaction failed");

    assert!(db.get::<Restaurant>("rest1").unwrap().is_some());
    assert!(db.get::<Order>("order1").unwrap().is_some());
}

#[test]
fn test_minidb_view_all() {
    let temp_file = NamedTempFile::new().expect("failed to create temp file");
    let db = MiniDB::builder(temp_file.path())
        .table::<Restaurant>()
        .build()
        .expect("failed to build store");

    let mut rests = Vec::new();
    for _ in 0..110 {
        rests.push(Restaurant { id: String::new() });
    }

    db.insert_many(&mut rests)
        .expect("failed to insert many restaurants");

    let first_five: Vec<Restaurant> = db
        .view_all::<Restaurant, _, _>(|iter| {
            iter.skip(100).take(5).collect::<Result<Vec<_>>>().unwrap()
        })
        .expect("failed to get first five");

    assert_eq!(first_five.len(), 5);
}
