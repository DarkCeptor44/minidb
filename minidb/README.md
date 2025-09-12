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

## Benchmarks

The benchmarks can be ran with:

```bash
cargo bench -p minidb
```

### Database

```text
Timer precision: 100 ns
database    fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ delete                 │               │               │               │         │
│  ├─ 1                   │               │               │               │         │
│  │  ├─ t=1  273.5 µs      │ 327.1 µs      │ 288.9 µs      │ 288.8 µs      │ 100     │ 100
│  │  ├─ t=4  477.9 µs      │ 1.368 ms      │ 915.8 µs      │ 887.8 µs      │ 100     │ 100
│  │  ├─ t=8  696.6 µs      │ 2.804 ms      │ 1.661 ms      │ 1.653 ms      │ 104     │ 104
│  │  ╰─ t=16  1.062 ms      │ 4.956 ms      │ 3.002 ms      │ 2.951 ms      │ 112     │ 112
│  ╰─ 1000                   │               │               │               │         │
│     ├─ t=1   274.3 ms      │ 298.7 ms      │ 283.9 ms      │ 285 ms        │ 100     │ 100
│     ├─ t=4   1.034 s       │ 1.119 s       │ 1.069 s       │ 1.068 s       │ 100     │ 100
│     ├─ t=8   2.035 s       │ 2.632 s       │ 2.128 s       │ 2.165 s       │ 104     │ 104
│     ╰─ t=16  4.339 s       │ 4.542 s       │ 4.4 s         │ 4.416 s       │ 112     │ 112
├─ exists                    │               │               │               │         │
│  ├─ 1                      │               │               │               │         │
│  │  ├─ t=1   91.49 µs      │ 191.8 µs      │ 92.69 µs      │ 94.87 µs      │ 100     │ 100
│  │  ├─ t=4   276.2 µs      │ 425.2 µs      │ 343.6 µs      │ 348 µs        │ 100     │ 100
│  │  ├─ t=8   479.3 µs      │ 774.1 µs      │ 613.5 µs      │ 607.9 µs      │ 104     │ 104
│  │  ╰─ t=16  714.5 µs      │ 1.31 ms       │ 1.038 ms      │ 1.024 ms      │ 112     │ 112
│  ╰─ 1000                   │               │               │               │         │
│     ├─ t=1   91.47 ms      │ 100.3 ms      │ 92.27 ms      │ 92.59 ms      │ 100     │ 100
│     ├─ t=4   188.3 ms      │ 217.5 ms      │ 195.7 ms      │ 198.2 ms      │ 100     │ 100
│     ├─ t=8   466.9 ms      │ 511.9 ms      │ 509 ms        │ 504.7 ms      │ 104     │ 104
│     ╰─ t=16  1.024 s       │ 1.056 s       │ 1.048 s       │ 1.044 s       │ 112     │ 112
├─ get                       │               │               │               │         │
│  ├─ 1                      │               │               │               │         │
│  │  ├─ t=1   205.5 µs      │ 508 µs        │ 207.8 µs      │ 213.9 µs      │ 100     │ 100
│  │  ├─ t=4   492.5 µs      │ 837.1 µs      │ 663.9 µs      │ 660.4 µs      │ 100     │ 100
│  │  ├─ t=8   635.2 µs      │ 1.518 ms      │ 1.279 ms      │ 1.237 ms      │ 104     │ 104
│  │  ╰─ t=16  1.039 ms      │ 2.566 ms      │ 2.297 ms      │ 2.113 ms      │ 112     │ 112
│  ╰─ 1000                   │               │               │               │         │
│     ├─ t=1   204.8 ms      │ 210.1 ms      │ 206.4 ms      │ 206.6 ms      │ 100     │ 100
│     ├─ t=4   376.5 ms      │ 418.2 ms      │ 392.6 ms      │ 392.8 ms      │ 100     │ 100
│     ├─ t=8   686.4 ms      │ 732.9 ms      │ 712 ms        │ 711.8 ms      │ 104     │ 104
│     ╰─ t=16  1.45 s        │ 1.587 s       │ 1.46 s        │ 1.478 s       │ 112     │ 112
├─ insert                    │               │               │               │         │
│  ├─ 1                      │               │               │               │         │
│  │  ├─ t=1   698.4 µs      │ 1.132 ms      │ 717.1 µs      │ 734.9 µs      │ 100     │ 100
│  │  ├─ t=4   919.5 µs      │ 3.753 ms      │ 2.177 ms      │ 2.099 ms      │ 100     │ 100
│  │  ├─ t=8   1.221 ms      │ 7.17 ms       │ 4.168 ms      │ 3.957 ms      │ 104     │ 104
│  │  ╰─ t=16  1.603 ms      │ 13.72 ms      │ 7.236 ms      │ 7.239 ms      │ 112     │ 112
│  ╰─ 1000                   │               │               │               │         │
│     ├─ t=1   724.4 ms      │ 1.527 s       │ 780.7 ms      │ 819.9 ms      │ 100     │ 100
│     ├─ t=4   2.975 s       │ 3.191 s       │ 3.091 s       │ 3.085 s       │ 100     │ 100
│     ├─ t=8   5.979 s       │ 6.386 s       │ 6.154 s       │ 6.152 s       │ 104     │ 104
│     ╰─ t=16  12.01 s       │ 12.63 s       │ 12.35 s       │ 12.33 s       │ 112     │ 112
├─ new                       │               │               │               │         │
│  ├─ t=1      1.485 ms      │ 1.787 ms      │ 1.534 ms      │ 1.54 ms       │ 100     │ 100
│  ├─ t=4      2.539 ms      │ 3.575 ms      │ 2.995 ms      │ 2.987 ms      │ 100     │ 100
│  ├─ t=8      4.216 ms      │ 5.654 ms      │ 5.088 ms      │ 4.999 ms      │ 104     │ 104
│  ╰─ t=16     7.947 ms      │ 10.85 ms      │ 9.929 ms      │ 9.728 ms      │ 112     │ 112
╰─ update                    │               │               │               │         │
   ├─ 1                      │               │               │               │         │
   │  ├─ t=1   746 µs        │ 6.473 ms      │ 800 µs        │ 886.6 µs      │ 100     │ 100
   │  ├─ t=4   1.042 ms      │ 12.66 ms      │ 2.802 ms      │ 2.858 ms      │ 100     │ 100
   │  ├─ t=8   1.165 ms      │ 7.75 ms       │ 4.052 ms      │ 4.146 ms      │ 104     │ 104
   │  ╰─ t=16  1.586 ms      │ 15.22 ms      │ 8.026 ms      │ 8.066 ms      │ 112     │ 112
   ╰─ 1000                   │               │               │               │         │
      ├─ t=1   771 ms        │ 839.2 ms      │ 796.9 ms      │ 799 ms        │ 100     │ 100
      ├─ t=4   3.232 s       │ 3.406 s       │ 3.282 s       │ 3.293 s       │ 100     │ 100
      ├─ t=8   6.704 s       │ 6.943 s       │ 6.748 s       │ 6.792 s       │ 104     │ 104
      ╰─ t=16  13.91 s       │ 14.37 s       │ 14.08 s       │ 14.09 s       │ 112     │ 112
```

### Id

```text
Timer precision: 100 ns
id              fastest       │ slowest       │ median        │ mean          │ samples │ iters
╰─ id_generate  2.299 µs      │ 160 µs        │ 2.399 µs      │ 3.979 µs      │ 100     │ 100
```

## License

This project is licensed under the [GNU Lesser General Public License v3](https://www.gnu.org/licenses/lgpl-3.0.en.html).
