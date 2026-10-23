# MiniDB

A minimal-but-functional structured wrapper for [redb](https://crates.io/crates/redb), using [Postcard](https://crates.io/crates/postcard) for serialization/deserialization.

## Crates

This workspace contains the following crates:

* **[minidb](./minidb/README.md)**: Main crate ([docs.rs](https://docs.rs/minidb))
* **[minidb-macros](./minidb-macros/README.md)**: Procedural macros for MiniDB

## Overview

MiniDB provides a structured interface around redb, handling serialization and deserialization of data types through Postcard and optional encryption with [XChaCha20Poly1305](https://crates.io/crates/chacha20poly1305). The main crate handles storage operations while the macros crate provides convenient derive macros for working with the database.

## Is This Vibecoded?

If by vibecoded you mean "I have no experience with coding and AI made this for me", then no. I have been coding for more than 12 years, 2 of them in Rust. I didn't let AI run free in this codebase, I had an idea, implemented a shittier version, and then asked AI to teach me how things could be better. The recently-developed IPC submodule though is mostly AI, because I had never messed with IPC before and needed a way to access the database from another process. Unit and integration tests show it running smoothly though. If you are skeptical about potential vulnerabilities, check out the [audits](./minidb/README.md#audits).

## License

This project is licensed under the Mozilla Public License, version 2.0. See the [LICENSE](LICENSE) file for details.
