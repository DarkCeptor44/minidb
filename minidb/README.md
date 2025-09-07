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

The database was initially built without async, then I thought about it and started writing async versions of each function in [minidb-utils](https://docs.rs/minidb-utils) but ultimately decided not to do it because there's no proper benchmark for concurrent async yet, the assumption is that the overhead wouldn't be worth it, and the API would be more complex, for example adding a table to the database instance would go from:

```rust
let db = Database::builder().path(path).table::<Person>().build().unwrap();
```

To:

```rust
let db = Database::builder().path(path).await.table::<Person>().await.build().await.unwrap();
```

However, it's not impossible if future benchmarks show enough difference.

## License

This project is licensed under the [GNU Lesser General Public License v3](https://www.gnu.org/licenses/lgpl-3.0.en.html).
