# minidb

Minimalistic file-based database written in Rust

## Features

* File-based, this means the tables are sub-directories and the records are files
* Uses [bitcode](https://crates.io/crates/bitcode) as the binary format to store the data
* Uses [cuid2](https://crates.io/crates/cuid2) slugs for record IDs
* Easy table definition with procedural macros
* Built around poison-free read-write locks to be thread-safe
* Relies on [serde](https://crates.io/crates/serde) for serialization and deserialization of the tables

## Why not async

The database was initially built without async, then I thought about it and wrote async versions of each filesystem-related function in [minidb-utils](https://docs.rs/minidb-utils) but ultimately decided not to do it because there's no proper benchmark for concurrent async yet, I'd assume the overhead from async wouldn't be worth it and the API would be more complex, for example adding a table to the database instance could go from:

```rust
let db = Database::builder().path(path).table::<Person>().build().unwrap();
```

To:

```rust
let db = Database::builder().path(path).await.table::<Person>().await.build().await.unwrap();
```

However, it's not impossible if future benchmarks show enough difference.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
minidb = "^0.1"
serde = { version = "^1", features = ["derive"] } 
```

## Usage

A minimal example of how to use minidb is provided in `examples/simple.rs`, you can run it with:

```bash
cargo run -p minidb --example simple

# or
cd minidb
cargo run --example simple
```

The example code:

```rust
use minidb::{AsTable, Database, Id, Table};
use serde::{Deserialize, Serialize};

#[derive(Debug, Table, Serialize, Deserialize, PartialEq)]
struct Person {
    #[key]
    id: Id<Self>,
    name: String,
    age: u8,
}

fn main() {
    // 1. Create database
    let db = Database::builder()
        .path("path/to/db")
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
```

## Tests

The tests can be ran with:

```bash
cargo test -p minidb

# or
cd minidb
cargo test
```

## License

This project is licensed under the [GNU Lesser General Public License v3](https://www.gnu.org/licenses/lgpl-3.0.en.html).
