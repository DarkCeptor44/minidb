use minidb::{AsTable, Database, Id, Table};
use serde::{Deserialize, Serialize};
use tempfile::tempdir;

#[derive(Debug, Table, Serialize, Deserialize, PartialEq)]
struct Person {
    #[key]
    id: Id<Self>,
    name: String,
    age: u8,
}

#[derive(Debug, Table, Serialize, Deserialize, PartialEq)]
struct Order {
    #[key]
    id: Id<Self>,
    price: f64,

    #[foreign_key]
    customer_id: Id<Person>,
}

fn main() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path();

    // 1. Create database
    let db = Database::builder()
        .path(db_path)
        .table::<Person>()
        .build()
        .unwrap();

    // 2. Insert a new person
    let mut person_to_insert = Person {
        id: Id::new(),
        name: "John Doe".to_string(),
        age: 31,
    };
    person_to_insert.id = db.insert(&person_to_insert).unwrap();
    println!("Inserted person: {:?}", person_to_insert);

    // 3. Insert a new order
    let mut order_to_insert = Order {
        id: Id::new(),
        price: 100.0,
        customer_id: person_to_insert.id.clone(),
    };
    order_to_insert.id = db.insert(&order_to_insert).unwrap();
    println!("Inserted order: {:?}", order_to_insert);

    // 4. Retrieve person
    let person_retrieved = db.get(&person_to_insert.id).unwrap();
    assert_eq!(person_retrieved, person_to_insert);
    println!(
        "Successfully retrieved and verified person: {:?}",
        person_retrieved
    );

    // 5. Retrieve order
    let order_retrieved = db.get(&order_to_insert.id).unwrap();
    assert_eq!(order_retrieved, order_to_insert);
    println!(
        "Successfully retrieved and verified order: {:?}",
        order_retrieved
    );

    // 6. Update person's age
    person_to_insert.age += 1;
    db.update(&person_to_insert).unwrap();
    println!("Updated person: {:?}", person_to_insert);

    // 7. Update order's price
    order_to_insert.price = 130.0;
    db.update(&order_to_insert).unwrap();
    println!("Updated order: {:?}", order_to_insert);

    // 8. Retrieve updated person
    let person_retrieved = db.get(&person_to_insert.id).unwrap();
    assert_eq!(person_retrieved.age, 32);
    println!(
        "Successfully retrieved and verified updated person: {:?}",
        person_retrieved
    );

    // 9. Retrieve updated order
    let order_retrieved = db.get(&order_to_insert.id).unwrap();
    assert_eq!(order_retrieved.price, 130.0);
    println!(
        "Successfully retrieved and verified updated order: {:?}",
        order_retrieved
    );

    // 10. Delete order
    db.delete(&order_to_insert.id).unwrap();
    println!("Deleted order");

    // 11. Delete person
    db.delete(&person_to_insert.id).unwrap();
    println!("Deleted person");

    // 12. Verify person is deleted
    let user_deleted = db.get(&person_to_insert.id);
    assert!(user_deleted.is_err());
    println!("Verified person deletion");

    // 13. Verify order is deleted
    let order_deleted = db.get(&order_to_insert.id);
    assert!(order_deleted.is_err());
    println!("Verified order deletion");

    println!("\nExample completed successfully");
}
