# minidb-macros

[![crates.io](https://img.shields.io/crates/v/minidb-macros.svg)](https://crates.io/crates/minidb-macros) [![docs](https://docs.rs/minidb-macros/badge.svg)](https://docs.rs/minidb-macros) [![MSRV](https://img.shields.io/crates/msrv/minidb-macros)](https://crates.io/crates/minidb-macros) [![license](https://img.shields.io/crates/l/minidb-macros.svg)](./LICENSE) [![downloads](https://img.shields.io/crates/d/minidb-macros)](https://crates.io/crates/minidb-macros)

[API Documentation](https://docs.rs/minidb) | [Workspace](../README.md)

Procedural macros for MiniDB

## Features

* Derive macros for MiniDB-compatible types
* Automatic implementation of necessary traits

## Is This Vibecoded?

If by vibecoded you mean "I have no experience with coding and AI made this for me", then no. I have been coding for more than 12 years, 2 of them in Rust. I didn't let AI run free in this codebase, I had an idea, implemented a shittier version, and then asked AI to teach me how things could be better. The recently-developed IPC submodule though is mostly AI, because I had never messed with IPC before and needed a way to access the database from another process. Unit and integration tests show it running smoothly though. If you are skeptical about potential vulnerabilities, check out the [audits](../minidb/README.md#audits).

## Usage

This crate is used as a dependency of the `minidb` crate

## License

This project is licensed under the Mozilla Public License, version 2.0. See the [LICENSE](LICENSE) file for details.
