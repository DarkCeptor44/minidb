# MiniDB

A minimal-but-functional structured wrapper for [redb](https://crates.io/crates/redb), using [Postcard](https://crates.io/crates/postcard) for serialization/deserialization.

## Crates

This workspace contains the following crates:

* **[minidb](./minidb/README.md)**: Main crate ([docs.rs](https://docs.rs/minidb))
* **[minidb-macros](./minidb-macros/README.md)**: Procedural macros for MiniDB

## Overview

MiniDB provides a structured interface around redb, handling serialization and deserialization of data types through Postcard and optional encryption with [XChaCha20Poly1305](https://crates.io/crates/chacha20poly1305). The main crate handles storage operations while the macros crate provides convenient derive macros for working with the database.

## License

This project is licensed under the Mozilla Public License, version 2.0. See the [LICENSE](LICENSE) file for details.
