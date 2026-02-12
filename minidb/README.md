# MiniDB

[API Documentation](https://docs.rs/minidb) | [Workspace](../README.md)

The main MiniDB crate providing a structured wrapper for [redb](https://crates.io/crates/redb) with serialization/deserialization.

## Features

* Automatic serialization/deserialization with [Postcard](https://crates.io/crates/postcard), using [serde](https://crates.io/crates/serde)
* Structured key-value storage
* Type-safe operations (mostly)
* Optional encryption using [XChaCha20Poly1305](https://crates.io/crates/chacha20poly1305)

## MSRV

| Version | MSRV |
|---------|------|
| 0.1.x   | 1.89 |

## Installation

In your `Cargo.toml`:

```toml
[dependencies]
minidb = { version = "0.1", features = ["macros"] } # or the latest version
serde = { version = "1", features = ["derive"] }  # needed for minidb
```

## Usage

Full examples can be found in the [examples](./examples) directory.

**Note:** The `#[derive(Table)]` macro requires the `macros` feature to be enabled.

```rust
#[derive(Table, Serialize, Deserialize)]
#[minidb(name = "people")]
struct Person {
   #[key]
   id: String,
   name: String,
   age: u8,
}

let db = MiniDB::builder("path/to/db")
      .table::<Person>()
      .build()
      .unwrap();

// insert a person
let mut p = Person {
   id: String::new(), // ID will be generated automatically, leave empty
   name: "John Doe".to_string(),
   age: 42,
};
db.insert(&mut p).unwrap();

// get a person by ID
let id = p.id.clone();
let new_person: Option<Person> = db.get(&id).unwrap();

if let Some(new_person) = new_person {
   println!("Found person: {}", new_person.name);
}
```

## Tests

Both integration and unit tests are included. They can be run with `cargo test`.

## Benchmarks

Benchmarks can be run with `cargo bench`, and the results look like this:

### Without Encryption

```text
Timer precision: 100 ns
minidb                        fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ all                                      │               │               │               │         │
│  ├─ 1000                    198.3 µs      │ 269.4 µs      │ 206.6 µs      │ 209.3 µs      │ 100     │ 100
│  ├─ 10000                   2.238 ms      │ 3.15 ms       │ 2.399 ms      │ 2.454 ms      │ 100     │ 100
│  ╰─ 100000                  23.11 ms      │ 26.53 ms      │ 23.8 ms       │ 23.96 ms      │ 100     │ 100
├─ create_table               389.7 µs      │ 1.029 ms      │ 441.1 µs      │ 486.7 µs      │ 100     │ 100
├─ export_table                             │               │               │               │         │
│  ├─ (1000, false)           243.8 µs      │ 455.1 µs      │ 246.3 µs      │ 270.2 µs      │ 100     │ 100
│  ├─ (1000, true)            250.2 µs      │ 327 µs        │ 252.6 µs      │ 255.5 µs      │ 100     │ 100
│  ├─ (10000, false)          2.456 ms      │ 3.621 ms      │ 2.921 ms      │ 2.895 ms      │ 100     │ 100
│  ╰─ (10000, true)           2.628 ms      │ 3.735 ms      │ 2.948 ms      │ 2.936 ms      │ 100     │ 100
├─ for_each                                 │               │               │               │         │
│  ├─ 1                       912.2 ns      │ 924.7 ns      │ 912.2 ns      │ 915.3 ns      │ 100     │ 1600
│  ├─ 1000                    169.9 µs      │ 185.5 µs      │ 171.7 µs      │ 172.5 µs      │ 100     │ 100
│  ╰─ 10000                   1.796 ms      │ 2.458 ms      │ 1.81 ms       │ 1.834 ms      │ 100     │ 100
├─ get                        799.7 ns      │ 818.5 ns      │ 806 ns        │ 805.6 ns      │ 100     │ 1600
├─ get (one from large db)                  │               │               │               │         │
│  ├─ 100                     943.5 ns      │ 1.374 µs      │ 949.7 ns      │ 956 ns        │ 100     │ 1600
│  ╰─ 1000                    1.037 µs      │ 1.062 µs      │ 1.043 µs      │ 1.044 µs      │ 100     │ 1600
├─ get_setting                699.7 ns      │ 1.212 µs      │ 712.2 ns      │ 718.9 ns      │ 100     │ 1600
├─ insert (existing db)       434.1 µs      │ 1.038 ms      │ 509.2 µs      │ 541.9 µs      │ 100     │ 100
├─ insert (fresh db)          435.6 µs      │ 1.533 ms      │ 482.7 µs      │ 531.9 µs      │ 100     │ 100
├─ insert_many (existing db)                │               │               │               │         │
│  ├─ 1                       483.4 µs      │ 1.036 ms      │ 595.9 µs      │ 622.5 µs      │ 100     │ 100
│  ├─ 1000                    3.831 ms      │ 29.8 ms       │ 18.69 ms      │ 16.93 ms      │ 100     │ 100
│  ╰─ 10000                   35.31 ms      │ 262 ms        │ 186 ms        │ 166.1 ms      │ 100     │ 100
├─ insert_many (fresh db)                   │               │               │               │         │
│  ├─ 1                       436.1 µs      │ 996.6 µs      │ 494.5 µs      │ 526.3 µs      │ 100     │ 100
│  ├─ 1000                    3.728 ms      │ 4.975 ms      │ 3.88 ms       │ 3.957 ms      │ 100     │ 100
│  ╰─ 10000                   34.67 ms      │ 38.88 ms      │ 35.27 ms      │ 35.55 ms      │ 100     │ 100
├─ is_empty                   643.5 ns      │ 662.2 ns      │ 649.7 ns      │ 651.4 ns      │ 100     │ 1600
├─ new                        5.551 ms      │ 9.073 ms      │ 6.196 ms      │ 6.509 ms      │ 100     │ 100
├─ remove (existing db)       443.8 µs      │ 904.2 µs      │ 467 µs        │ 504.9 µs      │ 100     │ 100
├─ remove (fresh db)          516.2 µs      │ 1.039 ms      │ 576 µs        │ 611.5 µs      │ 100     │ 100
├─ remove_many (existing db)                │               │               │               │         │
│  ├─ 1                       432.2 µs      │ 1.096 ms      │ 473.2 µs      │ 520 µs        │ 100     │ 100
│  ├─ 1000                    1.792 ms      │ 2.227 ms      │ 1.925 ms      │ 1.936 ms      │ 100     │ 100
│  ╰─ 10000                   16.41 ms      │ 22.08 ms      │ 17.12 ms      │ 17.22 ms      │ 100     │ 100
├─ remove_many (fresh db)                   │               │               │               │         │
│  ├─ 1                       2.461 ms      │ 4.425 ms      │ 2.792 ms      │ 2.877 ms      │ 100     │ 100
│  ├─ 1000                    3.745 ms      │ 6.356 ms      │ 4.287 ms      │ 4.419 ms      │ 100     │ 100
│  ╰─ 10000                   18.58 ms      │ 21.13 ms      │ 19.62 ms      │ 19.7 ms       │ 100     │ 100
├─ set_setting                460.3 µs      │ 919.9 µs      │ 516.5 µs      │ 552.6 µs      │ 100     │ 100
├─ update (existing db)       456.9 µs      │ 996.9 µs      │ 482.7 µs      │ 515.5 µs      │ 100     │ 100
├─ update (fresh db)          520.8 µs      │ 939.1 µs      │ 576.8 µs      │ 608 µs        │ 100     │ 100
├─ update_many (existing db)                │               │               │               │         │
│  ├─ 1                       458.9 µs      │ 955.2 µs      │ 496.1 µs      │ 530.1 µs      │ 100     │ 100
│  ├─ 1000                    1.661 ms      │ 25.14 ms      │ 17.64 ms      │ 14.79 ms      │ 100     │ 100
│  ╰─ 10000                   14.47 ms      │ 288.1 ms      │ 181.5 ms      │ 154.9 ms      │ 100     │ 100
╰─ update_many (fresh db)                   │               │               │               │         │
   ├─ 1                       534.4 µs      │ 948.8 µs      │ 623.9 µs      │ 648.9 µs      │ 100     │ 100
   ├─ 1000                    1.53 ms       │ 2.045 ms      │ 1.661 ms      │ 1.701 ms      │ 100     │ 100
   ╰─ 10000                   13.86 ms      │ 17.66 ms      │ 14.38 ms      │ 14.47 ms      │ 100     │ 100
```

### With Encryption

```text
Timer precision: 100 ns
encryption                    fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ all                                      │               │               │               │         │
│  ├─ 1000                    1.577 ms      │ 2.006 ms      │ 1.594 ms      │ 1.646 ms      │ 100     │ 100
│  ├─ 10000                   16.23 ms      │ 19.04 ms      │ 16.51 ms      │ 16.68 ms      │ 100     │ 100
│  ╰─ 100000                  164.9 ms      │ 192.6 ms      │ 184.1 ms      │ 180.2 ms      │ 100     │ 100
├─ create_table               427.3 µs      │ 992 µs        │ 485.9 µs      │ 500.8 µs      │ 100     │ 100
├─ export_table                             │               │               │               │         │
│  ├─ (1000, false)           1.719 ms      │ 2.008 ms      │ 1.87 ms       │ 1.875 ms      │ 100     │ 100
│  ├─ (1000, true)            1.71 ms       │ 2.033 ms      │ 1.861 ms      │ 1.87 ms       │ 100     │ 100
│  ├─ (10000, false)          18.04 ms      │ 20.7 ms       │ 19.5 ms       │ 19.47 ms      │ 100     │ 100
│  ╰─ (10000, true)           18.24 ms      │ 20.46 ms      │ 19.56 ms      │ 19.51 ms      │ 100     │ 100
├─ for_each                                 │               │               │               │         │
│  ├─ 1                       2.299 µs      │ 11.59 µs      │ 2.999 µs      │ 3.057 µs      │ 100     │ 100
│  ├─ 1000                    1.665 ms      │ 3.813 ms      │ 1.815 ms      │ 1.866 ms      │ 100     │ 100
│  ╰─ 10000                   17.19 ms      │ 19.35 ms      │ 17.94 ms      │ 17.92 ms      │ 100     │ 100
├─ get                        2.199 µs      │ 10.59 µs      │ 2.699 µs      │ 2.777 µs      │ 100     │ 100
├─ get (one from large db)                  │               │               │               │         │
│  ├─ 100                     2.399 µs      │ 12.99 µs      │ 3.299 µs      │ 3.338 µs      │ 100     │ 100
│  ╰─ 1000                    2.749 µs      │ 3.074 µs      │ 2.774 µs      │ 2.799 µs      │ 100     │ 400
├─ get_setting                2.037 µs      │ 5.912 µs      │ 2.256 µs      │ 2.334 µs      │ 100     │ 800
├─ insert (existing db)       544.3 µs      │ 1.264 ms      │ 654.4 µs      │ 700 µs        │ 100     │ 100
├─ insert (fresh db)          512.7 µs      │ 1.099 ms      │ 572.1 µs      │ 610 µs        │ 100     │ 100
├─ insert_many (existing db)                │               │               │               │         │
│  ├─ 1                       536.4 µs      │ 1.122 ms      │ 663 µs        │ 686 µs        │ 100     │ 100
│  ├─ 1000                    6.73 ms       │ 62.71 ms      │ 36.51 ms      │ 32.25 ms      │ 100     │ 100
│  ╰─ 10000                   68.31 ms      │ 491.5 ms      │ 358.3 ms      │ 322.2 ms      │ 100     │ 100
├─ insert_many (fresh db)                   │               │               │               │         │
│  ├─ 1                       503.5 µs      │ 1.626 ms      │ 601.5 µs      │ 639.8 µs      │ 100     │ 100
│  ├─ 1000                    6.078 ms      │ 7.5 ms        │ 6.754 ms      │ 6.782 ms      │ 100     │ 100
│  ╰─ 10000                   63.6 ms       │ 83.82 ms      │ 66.86 ms      │ 67.57 ms      │ 100     │ 100
├─ is_empty                   606 ns        │ 881 ns        │ 612.2 ns      │ 652 ns        │ 100     │ 1600
├─ new                        6.867 ms      │ 10.71 ms      │ 7.791 ms      │ 8.062 ms      │ 100     │ 100
├─ remove (existing db)       490.8 µs      │ 842 µs        │ 556.8 µs      │ 579.6 µs      │ 100     │ 100
├─ remove (fresh db)          620.2 µs      │ 1.163 ms      │ 736 µs        │ 763 µs        │ 100     │ 100
├─ remove_many (existing db)                │               │               │               │         │
│  ├─ 1                       491.2 µs      │ 1.071 ms      │ 563.6 µs      │ 604.3 µs      │ 100     │ 100
│  ├─ 1000                    3.603 ms      │ 4.308 ms      │ 3.92 ms       │ 3.923 ms      │ 100     │ 100
│  ╰─ 10000                   35.45 ms      │ 61.15 ms      │ 37.5 ms       │ 38.18 ms      │ 100     │ 100
├─ remove_many (fresh db)                   │               │               │               │         │
│  ├─ 1                       3.002 ms      │ 5.287 ms      │ 3.504 ms      │ 3.564 ms      │ 100     │ 100
│  ├─ 1000                    5.931 ms      │ 8.27 ms       │ 6.817 ms      │ 6.849 ms      │ 100     │ 100
│  ╰─ 10000                   38.94 ms      │ 55.53 ms      │ 42.19 ms      │ 42.62 ms      │ 100     │ 100
├─ set_setting                521.6 µs      │ 988.8 µs      │ 648.9 µs      │ 675.2 µs      │ 100     │ 100
├─ update (existing db)       531.6 µs      │ 980 µs        │ 628.2 µs      │ 657.4 µs      │ 100     │ 100
├─ update (fresh db)          618.8 µs      │ 1.251 ms      │ 719.9 µs      │ 743.5 µs      │ 100     │ 100
├─ update_many (existing db)                │               │               │               │         │
│  ├─ 1                       530.4 µs      │ 959.2 µs      │ 627 µs        │ 637.4 µs      │ 100     │ 100
│  ├─ 1000                    3.821 ms      │ 46.93 ms      │ 32.47 ms      │ 29.09 ms      │ 100     │ 100
│  ╰─ 10000                   39.27 ms      │ 508.9 ms      │ 355.5 ms      │ 305 ms        │ 100     │ 100
╰─ update_many (fresh db)                   │               │               │               │         │
   ├─ 1                       610.5 µs      │ 17.29 ms      │ 738.1 µs      │ 917.4 µs      │ 100     │ 100
   ├─ 1000                    3.577 ms      │ 4.351 ms      │ 3.955 ms      │ 3.944 ms      │ 100     │ 100
   ╰─ 10000                   36.43 ms      │ 60.65 ms      │ 38.72 ms      │ 41.19 ms      │ 100     │ 100
```

## License

This project is licensed under the [GNU Lesser General Public License v3](https://www.gnu.org/licenses/lgpl-3.0.en.html).
