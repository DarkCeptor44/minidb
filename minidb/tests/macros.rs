use minidb::{MiniDB, Table};
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

#[derive(Table, Serialize, Deserialize)]
#[minidb(name = "people")]
struct Person {
    #[key]
    id: String,
    name: String,
    age: u8,

    #[serde(skip)]
    ignored_field: bool,
}

#[test]
fn test_minidb_with_macros_insert() {
    let temp_file = NamedTempFile::new().expect("failed to create temp file");
    let db = MiniDB::builder(temp_file.path())
        .table::<Person>()
        .build()
        .expect("failed to create database");

    let mut p = Person {
        id: String::new(),
        name: "John Doe".to_string(),
        age: 31,
        ignored_field: true,
    };
    db.insert(&mut p).expect("failed to insert person");

    let all_people = db.all::<Person>().expect("failed to get all people");
    assert_eq!(all_people.len(), 1);

    let p = all_people.first().expect("person was not inserted");
    assert_eq!(p.name, "John Doe");
    assert_eq!(p.age, 31);
    assert!(!p.ignored_field);
}
