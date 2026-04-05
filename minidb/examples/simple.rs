use minidb::{
    MiniDB, TableModel,
    serde::{Deserialize, Serialize},
};
use tempfile::NamedTempFile;

// If you have the `macros` feature enabled, you can use the derive macro like this:

// #[derive(Debug, Table, Serialize, Deserialize, PartialEq)]
// #[minidb(name = "people")]
// struct Person {
//     #[key]
//     id: String,
//     name: String,
//     age: u8,
// }

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Person {
    id: String,
    name: String,
    age: u8,
}

impl TableModel for Person {
    const TABLE: redb::TableDefinition<'_, &'static str, &[u8]> =
        redb::TableDefinition::new("people");

    fn get_id(&self) -> &str {
        &self.id
    }

    fn set_id(&mut self, id: String) {
        self.id = id;
    }
}

fn main() {
    let temp_file = NamedTempFile::new().unwrap();

    // 1. Create a new database without encryption and only one table (Person)
    let db = MiniDB::builder(temp_file.path())
        .table::<Person>()
        .build()
        .unwrap();

    // 2. Insert a new person
    let mut person_to_insert = Person {
        id: String::new(), // this must be empty because MiniDB will generate an ID automatically
        name: "John Doe".to_string(),
        age: 31,
    };
    db.insert(&mut person_to_insert).unwrap();
    println!("Inserted person: {person_to_insert:?}");

    // 3. Retrieve person by ID
    let person_retrieved: Person = db.get(&person_to_insert.id).unwrap().unwrap();
    assert_eq!(person_retrieved, person_to_insert);
    println!("Successfully retrieved and verified person: {person_retrieved:?}");

    // 4. Update person
    person_to_insert.age += 1;
    db.update(&person_to_insert).unwrap();
    println!("Updated person: {person_to_insert:?}");

    // 5. Retrieve updated person
    let person_retrieved: Person = db.get(&person_to_insert.id).unwrap().unwrap();
    assert_eq!(person_retrieved.age, 32);
    println!("Successfully retrieved and verified updated person: {person_retrieved:?}");

    // 6. Delete person
    db.remove::<Person>(&person_to_insert.id).unwrap();
    println!("Deleted person");

    // 7. Verify person is deleted
    let user_deleted: Option<Person> = db.get(&person_to_insert.id).unwrap();
    assert!(user_deleted.is_none());
    println!("Verified deletion");

    println!("\nExample completed successfully");
}
