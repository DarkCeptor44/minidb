# minidb-utils

Utilities and fundamental functions for minidb. To be able to click on everything highlighted go to the [docs](https://docs.rs/crate/minidb-utils)

## Traits

* `IntoOptional`: Extension trait for `Option<T>`
* `PathExt`: Extension trait for any type that implements `AsRef<Path>` that adds some useful functions

## Structs

* `ArgonParams`: Struct to store Argon2 parameters that is easier to serialize/deserialize and pass it around

## Functions

**Note:** `async` functions are only available with the `tokio` feature

* `IntoOptional::into_optional`: Convert a value to an `Option<T>`

### Cryptographic

* `derive_key`: Derive a key from a password and a salt using [Argon2id](https://en.wikipedia.org/wiki/Argon2)

* `generate_salt`: Generate a random salt of 16 bytes
* `hash_password`: Hash a password using [Argon2id](https://en.wikipedia.org/wiki/Argon2)
* `verify_password`: Verify a password using [Argon2id](https://en.wikipedia.org/wiki/Argon2)

### File related

* `deserialize_file`: Deserialize [bitcode] data from a file

* `deserialize_file_async`: Deserialize [bitcode] data from a file asynchronously
* `read_from_file`: Read a file into a string using a buffer
* `read_from_file_async`: Read a file asynchronously into a string using a buffer
* `serialize_file`: Serialize [bitcode] data to a file
* `serialize_file_async`: Serialize [bitcode] data to a file asynchronously

### Path related

* `PathExt::is_empty`: Check if a path is a directory and empty

## MSRV

The minimum supported Rust version is `1.85.0`. The MSRV might be changed at any time with a minor version bump

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
cargo test -p minidb-utils --all-features
```

## Benchmarks

### Cryptographic

The tuple in [`derive_key` is `(memory_cost, iterations, parallelism)`, where:

*Memory cost of 1024 is benchmarked but is not recommended

* 19 MiB is recommended with `T` of 2 or 3 and `P` of 1 and is the default of [Argon2](https://en.wikipedia.org/wiki/Argon2)
* 64 MiB with `T` of 3 and `P` of 2 is used by [Bitwarden](https://bitwarden.com)

```text
Timer precision: 100 ns
crypto               fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ derive_key                      │               │               │               │         │
│  ├─ (1024, 2, 1)                 │               │               │               │         │
│  │  ├─ t=1         880.6 µs      │ 1.47 ms       │ 928.2 µs      │ 999.7 µs      │ 100     │ 100
│  │  ├─ t=4         960.3 µs      │ 1.83 ms       │ 1.312 ms      │ 1.334 ms      │ 100     │ 100
│  │  ├─ t=8         1.037 ms      │ 2.589 ms      │ 1.584 ms      │ 1.582 ms      │ 104     │ 104
│  │  ╰─ t=16        1.288 ms      │ 3.886 ms      │ 2.032 ms      │ 2.197 ms      │ 112     │ 112
│  ├─ (19456, 3, 1)                │               │               │               │         │
│  │  ├─ t=1         25.87 ms      │ 36.72 ms      │ 29.67 ms      │ 29.76 ms      │ 100     │ 100
│  │  ├─ t=4         31.97 ms      │ 47.31 ms      │ 39.95 ms      │ 39.76 ms      │ 100     │ 100
│  │  ├─ t=8         49.93 ms      │ 74.86 ms      │ 64.52 ms      │ 64.68 ms      │ 104     │ 104
│  │  ╰─ t=16        81.58 ms      │ 146.5 ms      │ 110.4 ms      │ 109.3 ms      │ 112     │ 112
│  ╰─ (65536, 3, 2)                │               │               │               │        │
│     ├─ t=1         97.89 ms      │ 125.5 ms      │ 107.2 ms      │ 107.7 ms      │ 100     │ 100
│     ├─ t=4         131.6 ms      │ 168.3 ms      │ 149.6 ms      │ 150.9 ms      │ 100     │ 100
│     ├─ t=8         211.7 ms      │ 275.6 ms      │ 235.5 ms      │ 236 ms        │ 104     │ 104
│     ╰─ t=16        299.9 ms      │ 503.7 ms      │ 380 ms        │ 374.7 ms      │ 112     │ 112
├─ generate_salt                   │               │               │               │         │
│  ├─ t=1            41.55 ns      │ 55.61 ns      │ 41.94 ns      │ 43.69 ns      │ 100     │ 25600
│  ├─ t=4            42.72 ns      │ 73.97 ns      │ 63.43 ns      │ 58.94 ns      │ 100     │ 25600
│  ├─ t=8            43.51 ns      │ 123.9 ns      │ 66.94 ns      │ 69.01 ns      │ 104     │ 13312
│  ╰─ t=16           44.29 ns      │ 110.6 ns      │ 67.72 ns      │ 69.21 ns      │ 112     │ 14336
├─ hash_password                   │               │               │               │         │
│  ├─ (1024, 2, 1)                 │               │               │               │         │
│  │  ├─ t=1         900.5 µs      │ 1.474 ms      │ 1.076 ms      │ 1.076 ms      │ 100     │ 100
│  │  ├─ t=4         1.003 ms      │ 2.261 ms      │ 1.265 ms      │ 1.293 ms      │ 100     │ 100
│  │  ├─ t=8         1.052 ms      │ 2.198 ms      │ 1.649 ms      │ 1.636 ms      │ 104     │ 104
│  │  ╰─ t=16        1.337 ms      │ 4.191 ms      │ 1.974 ms      │ 2.226 ms      │ 112     │ 112
│  ├─ (19456, 3, 1)                │               │               │               │         │
│  │  ├─ t=1         27.44 ms      │ 44.44 ms      │ 31.73 ms      │ 31.88 ms      │ 100     │ 100
│  │  ├─ t=4         36.61 ms      │ 52.14 ms      │ 44.83 ms      │ 45.03 ms      │ 100     │ 100
│  │  ├─ t=8         55.24 ms      │ 90.34 ms      │ 67.9 ms       │ 67.88 ms      │ 104     │ 104
│  │  ╰─ t=16        79.98 ms      │ 133.6 ms      │ 112.3 ms      │ 109.7 ms      │ 112     │ 112
│  ╰─ (65536, 3, 2)                │               │               │               │        │
│     ├─ t=1         101.9 ms      │ 156.8 ms      │ 113.3 ms      │ 113.8 ms      │ 100     │ 100
│     ├─ t=4         134.7 ms      │ 191.6 ms      │ 151.3 ms      │ 155 ms        │ 100     │ 100
│     ├─ t=8         210.5 ms      │ 269.5 ms      │ 234.9 ms      │ 236.6 ms      │ 104     │ 104
│     ╰─ t=16        305.5 ms      │ 451.8 ms      │ 378.1 ms      │ 380.8 ms      │ 112     │ 112
╰─ verify_password                 │               │               │               │         │
   ├─ (1024, 2, 1)                 │               │               │               │         │
   │  ├─ t=1         890.8 µs      │ 1.542 ms      │ 1.066 ms      │ 1.076 ms      │ 100     │ 100
   │  ├─ t=4         939.8 µs      │ 1.987 ms      │ 1.192 ms      │ 1.285 ms      │ 100     │ 100
   │  ├─ t=8         1.018 ms      │ 3.147 ms      │ 1.629 ms      │ 1.631 ms      │ 104     │ 104
   │  ╰─ t=16        1.346 ms      │ 3.713 ms      │ 2.03 ms       │ 2.173 ms      │ 112     │ 112
   ├─ (19456, 3, 1)                │               │               │               │         │
   │  ├─ t=1         25.44 ms      │ 33.45 ms      │ 28.51 ms      │ 28.39 ms      │ 100     │ 100
   │  ├─ t=4         30.83 ms      │ 49.71 ms      │ 40.55 ms      │ 40.61 ms      │ 100     │ 100
   │  ├─ t=8         54.29 ms      │ 82.59 ms      │ 66.05 ms      │ 66.82 ms      │ 104     │ 104
   │  ╰─ t=16        78.09 ms      │ 134.8 ms      │ 104.2 ms      │ 104.1 ms      │ 112     │ 112
   ╰─ (65536, 3, 2)                │               │               │               │         │
      ├─ t=1         100.9 ms      │ 138.1 ms      │ 107.6 ms      │ 108.6 ms      │ 100     │ 100
      ├─ t=4         136.6 ms      │ 175.1 ms      │ 152 ms        │ 153.6 ms      │ 100     │ 100
      ├─ t=8         208.3 ms      │ 254.8 ms      │ 229.5 ms      │ 229.3 ms      │ 104     │ 104
      ╰─ t=16        287.7 ms      │ 441.4 ms      │ 365.6 ms      │ 367.7 ms      │ 112     │ 112
```

### File related

```text
Timer precision: 100 ns
fs                         fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ deserialize_file        734.7 µs      │ 1.499 ms      │ 952.7 µs      │ 967.7 µs      │ 100     │ 100
├─ deserialize_file_async  758.7 µs      │ 1.702 ms      │ 972.3 µs      │ 1.005 ms      │ 100     │ 100
├─ read_from_file          654.4 µs      │ 1.347 ms      │ 791 µs        │ 834.5 µs      │ 100     │ 100
├─ read_from_file_async    706.8 µs      │ 1.844 ms      │ 831.9 µs      │ 882.3 µs      │ 100     │ 100
├─ serialize_file          723.1 µs      │ 1.108 ms      │ 779.8 µs      │ 803.1 µs      │ 100     │ 100
╰─ serialize_file_async    783 µs        │ 1.267 ms      │ 885.4 µs      │ 912.9 µs      │ 100     │ 100
```

### Path related

```text
Timer precision: 100 ns
path         fastest       │ slowest       │ median        │ mean          │ samples │ iters
╰─ is_empty  222.6 µs      │ 325.1 µs      │ 231.9 µs      │ 240.6 µs      │ 100     │ 100
```

### Binary formats

```text
Timer precision: 100 ns
formats            fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ bitcode                       │               │               │               │         │
│  ├─ deserialize  541.9 µs      │ 2.083 ms      │ 607.2 µs      │ 688.8 µs      │ 100     │ 100
│  ╰─ serialize    137.6 µs      │ 1.635 ms      │ 171.4 µs      │ 194.3 µs      │ 100     │ 100
├─ minicbor                      │               │               │               │         │
│  ├─ deserialize  1.104 ms      │ 2.641 ms      │ 1.227 ms      │ 1.314 ms      │ 100     │ 100
│  ╰─ serialize    149.2 µs      │ 528.8 µs      │ 261.2 µs      │ 260.1 µs      │ 100     │ 100
├─ postcard                      │               │               │               │         │
│  ├─ deserialize  485 µs        │ 1.536 ms      │ 509.9 µs      │ 616 µs        │ 100     │ 100
│  ╰─ serialize    66.39 µs      │ 149 µs        │ 66.59 µs      │ 73.91 µs      │ 100     │ 100
├─ rmp_serde                     │               │               │               │         │
│  ├─ deserialize  779.8 µs      │ 4.313 ms      │ 1.237 ms      │ 1.31 ms       │ 100     │ 100
│  ╰─ serialize    91.19 µs      │ 229.4 µs      │ 91.54 µs      │ 93.73 µs      │ 100     │ 100
╰─ speedy_vec                    │               │               │               │         │
   ├─ deserialize  397.5 µs      │ 2.375 ms      │ 592.8 µs      │ 649.8 µs      │ 100     │ 100
   ╰─ serialize    32.59 µs      │ 112.9 µs      │ 62.89 µs      │ 64.02 µs      │ 100     │ 100
```

## License

This project is licensed under the [GNU Lesser General Public License v3](https://www.gnu.org/licenses/lgpl-3.0.en.html).
