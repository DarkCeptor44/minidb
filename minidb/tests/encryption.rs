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

#[test]
fn test_encrypted_database_new() {
    let pass = "secretpassword123";
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let temp_path = temp_dir.path();
    let db = Database::builder()
        .path(temp_path)
        .encryption(pass)
        .table::<Person>()
        .build()
        .expect("Failed to build database");

    assert!(dbg!(db).path().is_dir());
    assert!(temp_path.join(Person::name()).is_dir());
}
