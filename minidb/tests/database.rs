use minidb::{AsTable, Database, Id, Table};
use minidb_utils::read_from_file;
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
fn test_database_new() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let temp_path = temp_dir.path();
    let db = Database::builder()
        .path(temp_path)
        .table::<Person>()
        .build()
        .expect("Failed to build database");

    assert!(dbg!(db).path().is_dir());
    assert!(temp_path.join(Person::name()).is_dir());
}

#[test]
fn test_database_add_record() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let temp_path = temp_dir.path();
    let db = Database::builder()
        .path(temp_path)
        .table::<Person>()
        .build()
        .expect("Failed to build database");

    let mut p = Person {
        id: Id::default(),
        name: String::from("John Doe"),
        age: 31,
    };
    let id = p.insert(&dbg!(db)).expect("Failed to insert person");
    p.id = id;

    let str = read_from_file(temp_path.join("person").join(&p.id)).expect("Failed to read file");
    assert_eq!(str, format!("\0\u{8}John Doe\u{1f}"));
}

#[test]
fn test_database_get_record() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let temp_path = temp_dir.path();
    let db = Database::builder()
        .path(temp_path)
        .table::<Person>()
        .build()
        .expect("Failed to create database");

    let mut p = Person {
        id: Id::new(),
        name: String::from("John Doe"),
        age: 31,
    };

    let id = p.insert(dbg!(&db)).expect("Failed to insert person");
    p.id = id;

    let p2 = Person::get(&db, dbg!(&p.id)).expect("Failed to get person");
    assert_eq!(p2, p);
}

#[test]
fn test_database_update_record() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let temp_path = temp_dir.path();
    let db = Database::builder()
        .path(temp_path)
        .table::<Person>()
        .build()
        .expect("Failed to create database");

    let mut p = Person {
        id: Id::new(),
        name: String::from("John Doe"),
        age: 31,
    };

    let id = p.insert(dbg!(&db)).expect("Failed to insert person");
    p.id = id;

    p.age += 1;
    p.update(&db).expect("Failed to update person");

    let p2 = Person::get(&db, &p.id).expect("Failed to get person");
    assert_eq!(p2, p);
}

#[test]
fn test_database_delete_record() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let temp_path = temp_dir.path();
    let db = Database::builder()
        .path(temp_path)
        .table::<Person>()
        .build()
        .expect("Failed to create database");

    let p = Person {
        id: Id::new(),
        name: String::from("John Doe"),
        age: 31,
    };

    let id = p.insert(dbg!(&db)).expect("Failed to insert person");

    Person::delete(&db, dbg!(&id)).expect("Failed to delete person");

    assert!(Person::get(&db, &id).is_err());
}

#[test]
fn test_database_macros() {
    #![allow(dead_code)]

    struct NotTable1;

    #[derive(Table, Serialize, Deserialize)]
    struct NotTable2 {
        #[key]
        id: Id<Self>,
    }

    #[derive(Table, Serialize, Deserialize)]
    #[minidb(name = "people")]
    struct PersonTest {
        #[key]
        id: Id<Self>,

        name: String,

        age: Age,

        #[serde(skip)]
        other_name: String,
    }

    #[derive(Serialize, Deserialize)]
    struct Age(u8);

    assert_eq!(NotTable2::name(), "not_table2");
    assert_eq!(PersonTest::name(), "people");
}

#[test]
fn test_database_relationship() {
    #[derive(Debug, Table, Serialize, Deserialize)]
    struct Order {
        #[key]
        id: Id<Self>,

        #[foreign_key]
        customer_id: Id<Person>,
    }

    let temp_dir = tempdir().expect("Failed to create temp dir");
    let temp_path = temp_dir.path();
    let db = Database::builder()
        .path(temp_path)
        .table::<Person>()
        .table::<Order>()
        .build()
        .expect("Failed to build database");

    dbg!(&db);

    let mut p = Person {
        id: Id::new(),
        name: "John Doe".into(),
        age: 31,
    };

    assert_eq!(Order::get_foreign_keys().len(), 1);

    p.id = p.insert(&db).expect("Failed to insert person");
    dbg!(&p);

    let mut o = Order {
        id: Id::new(),
        customer_id: p.id.clone(),
    };

    o.id = o.insert(&db).expect("Failed to insert order");
    dbg!(&o);

    assert_eq!(o.customer_id, p.id);

    p.age = 32;
    p.update(&db).expect("Failed to update person");

    let p2 = Person::get(&db, &p.id).expect("Failed to get person");
    assert_eq!(p2, p);

    Order::delete(&db, &o.id).expect("Failed to delete order");
    Person::delete(&db, &p.id).expect("Failed to delete person");
}
