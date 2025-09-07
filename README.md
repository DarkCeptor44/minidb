# minidb

Minimalistic file-based database written in Rust

## Crates in this workspace

* **[minidb](./minidb/README.md)**: Main crate
* **[minidb-macros](./minidb-macros/README.md)**: Contains procedural macros for defining the tables
* **[minidb-utils](./minidb-utils/README.md)**: Contains utilities and fundamental functions for minidb

## Audits

* From [cargo-audit](https://crates.io/crates/cargo-audit):

```text
Crate:     atomic-polyfill
Version:   1.0.3
Warning:   unmaintained
Title:     atomic-polyfill is unmaintained
Date:      2023-07-11
ID:        RUSTSEC-2023-0089
URL:       https://rustsec.org/advisories/RUSTSEC-2023-0089
Dependency tree:
atomic-polyfill 1.0.3
└── heapless 0.7.17
    └── postcard 1.1.3
        └── minidb-utils 0.1.0
            └── minidb 0.1.0

Crate:     paste
Version:   1.0.15
Warning:   unmaintained
Title:     paste - no longer maintained
Date:      2024-10-07
ID:        RUSTSEC-2024-0436
URL:       https://rustsec.org/advisories/RUSTSEC-2024-0436
Dependency tree:
paste 1.0.15
├── rmp 0.8.14
│   └── rmp-serde 1.3.0
│       └── minidb-utils 0.1.0
│           └── minidb 0.1.0
└── minidb 0.1.0

Crate:     serde_cbor
Version:   0.11.2
Warning:   unmaintained
Title:     serde_cbor is unmaintained
Date:      2021-08-15
ID:        RUSTSEC-2021-0127
URL:       https://rustsec.org/advisories/RUSTSEC-2021-0127
Dependency tree:
serde_cbor 0.11.2
└── minidb-utils 0.1.0
    └── minidb 0.1.0

warning: 3 allowed warnings found
```

## Tests

The tests can be run with:

```bash
cargo test --workspace --all-features
```

## License

This project is licensed under the [GNU Lesser General Public License v3](https://www.gnu.org/licenses/lgpl-3.0.en.html).
