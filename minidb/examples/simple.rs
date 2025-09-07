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
    let id = db.insert(&person_to_insert).unwrap();
    person_to_insert.id = id;
    println!("Inserted person: {:?}", person_to_insert);

    // 3. Retrieve person
    let person_retrieved = db.get(&person_to_insert.id).unwrap();
    assert_eq!(person_retrieved, person_to_insert);
    println!(
        "Successfully retrieved and verified person: {:?}",
        person_retrieved
    );

    // 4. Update person's age
    person_to_insert.age += 1;
    db.update(&person_to_insert).unwrap();
    println!("Updated person: {:?}", person_to_insert);

    // 5. Retrieve updated person
    let person_retrieved = db.get(&person_to_insert.id).unwrap();
    assert_eq!(person_retrieved.age, 32);
    println!(
        "Successfully retrieved and verified updated person: {:?}",
        person_retrieved
    );

    // 6. Delete person
    db.delete(&person_to_insert.id).unwrap();
    println!("Deleted person");

    // 7. Verify person is deleted
    let user_deleted = db.get(&person_to_insert.id);
    assert!(user_deleted.is_err());
    println!("Verified deletion");

    println!("\nExample completed successfully");
}
