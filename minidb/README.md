# MiniDB

[![crates.io](https://img.shields.io/crates/v/minidb.svg)](https://crates.io/crates/minidb) [![docs](https://docs.rs/minidb/badge.svg)](https://docs.rs/minidb) [![MSRV](https://img.shields.io/crates/msrv/minidb)](https://crates.io/crates/minidb) [![license](https://img.shields.io/crates/l/minidb.svg)](./LICENSE) [![downloads](https://img.shields.io/crates/d/minidb)](https://crates.io/crates/minidb)

[API Documentation](https://docs.rs/minidb) | [Workspace](../README.md)

The main MiniDB crate providing a structured wrapper for [redb](https://crates.io/crates/redb) with serialization/deserialization.

## Key Features

* ACID compliant and whatever else [redb](https://crates.io/crates/redb) claims
* Automatic serialization/deserialization with [Postcard](https://crates.io/crates/postcard), using [serde](https://crates.io/crates/serde)
* Structured key-value storage with automatic [CUID2](https://crates.io/crates/cuid2) IDs
* Type-safe operations (mostly)
* Optional encryption using [XChaCha20Poly1305](https://crates.io/crates/chacha20poly1305)
* Includes derive macros (e.g., `#[derive(Table)]`) for easy table definition
* "Relational" (requires manual management of foreign keys)
* IPC fallback with helper server

## Is This Vibecoded?

If by vibecoded you mean "I have no experience with coding and AI made this for me", then no. I have been coding for more than 12 years, 2 of them in Rust. I didn't let AI run free in this codebase, I had an idea, implemented a shittier version, and then asked AI to teach me how things could be better. The recently-developed IPC submodule though is mostly AI, because I had never messed with IPC before and needed a way to access the database from another process. Unit and integration tests show it running smoothly though. If you are skeptical about potential vulnerabilities, check out the [audits](#audits).

## MSRV

| Version | MSRV | Edition |
| --- | --- | --- |
| <= 0.5.0 | 1.90 | 2024 |

## Installation

In your `Cargo.toml`:

```toml
[dependencies]
minidb = { version = "0.5.0", features = ["macros"] } # or whatever the latest version is
serde = { version = "1.0.229", features = ["derive"] }
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

let db = MiniDB::builder().path("path/to/db")
      .table::<Person>()
      .open()
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

## Audits

| **Auditor** | **Audit Date** | **Version** | **Vulnerabilities** |
| --- | --- | --- | --- |
| [cargo-audit](https://crates.io/crates/cargo-audit) | 2026-08-01 | 0.5.0 | 1 ([`atomic-polyfill`](https://rustsec.org/advisories/RUSTSEC-2023-0089) - unmaintained) |

* I personally don't consider unmaintained crates that big of an issue, but if `postcard` ever updates its version of `heapless` then I'll update `postcard`

## Tests

Both integration and unit tests are included. They can be run with `cargo test --all-features`.

## Benchmarks

Benchmarks can be found in [benchmarks](./docs/benchmarks.md).

## License

This project is licensed under the Mozilla Public License, version 2.0. See the [LICENSE](LICENSE) file for details.
