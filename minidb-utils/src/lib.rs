// Copyright (c) 2025, DarkCeptor44
//
// This file is licensed under the GNU Lesser General Public License
// (either version 3 or, at your option, any later version).
//
// This software comes without any warranty, express or implied. See the
// GNU Lesser General Public License for details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with this software. If not, see <https://www.gnu.org/licenses/>.

//! # Utilities
//!
//! Useful functions for minidb
//!
//! ## Traits
//!
//! * [`IntoOptional`]: Extension trait for [`Option<T>`]
//! * [`PathExt`]: Extension trait for any type that implements [`AsRef<Path>`] that adds some useful functions
//!
//! ## Structs
//!
//! * [`ArgonParams`]: Struct to store Argon2 parameters that is easier to serialize/deserialize and pass it around
//!
//! ## Functions
//!
//! **Note:** `async` functions are only available with the `tokio` feature
//!
//! * [`IntoOptional::into_optional`]: Convert a value to an [`Option<T>`]
//!
//! ### Cryptographic
//!
//! * [`derive_key`]: Derive a key from a password and a salt using [Argon2id](argon2)
//! * [`generate_salt`]: Generate a random salt of 16 bytes
//! * [`hash_password`]: Hash a password using [Argon2id](argon2)
//! * [`verify_password`]: Verify a password using [Argon2id](argon2)
//!
//! ### File related
//!
//! * [`deserialize_file`]: Deserialize [bitcode] data from a file
//! * [`deserialize_file_async`]: Deserialize [bitcode] data from a file asynchronously
//! * [`read_from_file`]: Read a file into a string using a buffer
//! * [`read_from_file_async`]: Read a file asynchronously into a string using a buffer
//! * [`serialize_file`]: Serialize [bitcode] data to a file
//! * [`serialize_file_async`]: Serialize [bitcode] data to a file asynchronously
//!
//! ### Path related
//!
//! * [`PathExt::is_empty`]: Check if a path is a directory and empty
//!
//! ## Benchmarks
//!
//! ### Cryptographic
//!
//! The tuple in [`derive_key`] is `(memory_cost, iterations, parallelism)`, where:
//!
//! * Memory cost of 1024 is benchmarked but is not recommended
//! * 19 MiB is recommended with `T` of 2 or 3 and `P` of 1 and is the default of [Argon2](argon2)
//! * 64 MiB with `T` of 3 and `P` of 2 is used by [Bitwarden](https://bitwarden.com)
//!
//! ```text
//! Timer precision: 100 ns
//! crypto               fastest       │ slowest       │ median        │ mean          │ samples │ iters
//! ├─ derive_key                      │               │               │               │         │
//! │  ├─ (1024, 2, 1)                 │               │               │               │         │
//! │  │  ├─ t=1         880.6 µs      │ 1.47 ms       │ 928.2 µs      │ 999.7 µs      │ 100     │ 100
//! │  │  ├─ t=4         960.3 µs      │ 1.83 ms       │ 1.312 ms      │ 1.334 ms      │ 100     │ 100
//! │  │  ├─ t=8         1.037 ms      │ 2.589 ms      │ 1.584 ms      │ 1.582 ms      │ 104     │ 104
//! │  │  ╰─ t=16        1.288 ms      │ 3.886 ms      │ 2.032 ms      │ 2.197 ms      │ 112     │ 112
//! │  ├─ (19456, 3, 1)                │               │               │               │         │
//! │  │  ├─ t=1         25.87 ms      │ 36.72 ms      │ 29.67 ms      │ 29.76 ms      │ 100     │ 100
//! │  │  ├─ t=4         31.97 ms      │ 47.31 ms      │ 39.95 ms      │ 39.76 ms      │ 100     │ 100
//! │  │  ├─ t=8         49.93 ms      │ 74.86 ms      │ 64.52 ms      │ 64.68 ms      │ 104     │ 104
//! │  │  ╰─ t=16        81.58 ms      │ 146.5 ms      │ 110.4 ms      │ 109.3 ms      │ 112     │ 112
//! │  ╰─ (65536, 3, 2)                │               │               │               │        │
//! │     ├─ t=1         97.89 ms      │ 125.5 ms      │ 107.2 ms      │ 107.7 ms      │ 100     │ 100
//! │     ├─ t=4         131.6 ms      │ 168.3 ms      │ 149.6 ms      │ 150.9 ms      │ 100     │ 100
//! │     ├─ t=8         211.7 ms      │ 275.6 ms      │ 235.5 ms      │ 236 ms        │ 104     │ 104
//! │     ╰─ t=16        299.9 ms      │ 503.7 ms      │ 380 ms        │ 374.7 ms      │ 112     │ 112
//! ├─ generate_salt                   │               │               │               │         │
//! │  ├─ t=1            41.55 ns      │ 55.61 ns      │ 41.94 ns      │ 43.69 ns      │ 100     │ 25600
//! │  ├─ t=4            42.72 ns      │ 73.97 ns      │ 63.43 ns      │ 58.94 ns      │ 100     │ 25600
//! │  ├─ t=8            43.51 ns      │ 123.9 ns      │ 66.94 ns      │ 69.01 ns      │ 104     │ 13312
//! │  ╰─ t=16           44.29 ns      │ 110.6 ns      │ 67.72 ns      │ 69.21 ns      │ 112     │ 14336
//! ├─ hash_password                   │               │               │               │         │
//! │  ├─ (1024, 2, 1)                 │               │               │               │         │
//! │  │  ├─ t=1         900.5 µs      │ 1.474 ms      │ 1.076 ms      │ 1.076 ms      │ 100     │ 100
//! │  │  ├─ t=4         1.003 ms      │ 2.261 ms      │ 1.265 ms      │ 1.293 ms      │ 100     │ 100
//! │  │  ├─ t=8         1.052 ms      │ 2.198 ms      │ 1.649 ms      │ 1.636 ms      │ 104     │ 104
//! │  │  ╰─ t=16        1.337 ms      │ 4.191 ms      │ 1.974 ms      │ 2.226 ms      │ 112     │ 112
//! │  ├─ (19456, 3, 1)                │               │               │               │         │
//! │  │  ├─ t=1         27.44 ms      │ 44.44 ms      │ 31.73 ms      │ 31.88 ms      │ 100     │ 100
//! │  │  ├─ t=4         36.61 ms      │ 52.14 ms      │ 44.83 ms      │ 45.03 ms      │ 100     │ 100
//! │  │  ├─ t=8         55.24 ms      │ 90.34 ms      │ 67.9 ms       │ 67.88 ms      │ 104     │ 104
//! │  │  ╰─ t=16        79.98 ms      │ 133.6 ms      │ 112.3 ms      │ 109.7 ms      │ 112     │ 112
//! │  ╰─ (65536, 3, 2)                │               │               │               │        │
//! │     ├─ t=1         101.9 ms      │ 156.8 ms      │ 113.3 ms      │ 113.8 ms      │ 100     │ 100
//! │     ├─ t=4         134.7 ms      │ 191.6 ms      │ 151.3 ms      │ 155 ms        │ 100     │ 100
//! │     ├─ t=8         210.5 ms      │ 269.5 ms      │ 234.9 ms      │ 236.6 ms      │ 104     │ 104
//! │     ╰─ t=16        305.5 ms      │ 451.8 ms      │ 378.1 ms      │ 380.8 ms      │ 112     │ 112
//! ╰─ verify_password                 │               │               │               │         │
//!    ├─ (1024, 2, 1)                 │               │               │               │         │
//!    │  ├─ t=1         890.8 µs      │ 1.542 ms      │ 1.066 ms      │ 1.076 ms      │ 100     │ 100
//!    │  ├─ t=4         939.8 µs      │ 1.987 ms      │ 1.192 ms      │ 1.285 ms      │ 100     │ 100
//!    │  ├─ t=8         1.018 ms      │ 3.147 ms      │ 1.629 ms      │ 1.631 ms      │ 104     │ 104
//!    │  ╰─ t=16        1.346 ms      │ 3.713 ms      │ 2.03 ms       │ 2.173 ms      │ 112     │ 112
//!    ├─ (19456, 3, 1)                │               │               │               │         │
//!    │  ├─ t=1         25.44 ms      │ 33.45 ms      │ 28.51 ms      │ 28.39 ms      │ 100     │ 100
//!    │  ├─ t=4         30.83 ms      │ 49.71 ms      │ 40.55 ms      │ 40.61 ms      │ 100     │ 100
//!    │  ├─ t=8         54.29 ms      │ 82.59 ms      │ 66.05 ms      │ 66.82 ms      │ 104     │ 104
//!    │  ╰─ t=16        78.09 ms      │ 134.8 ms      │ 104.2 ms      │ 104.1 ms      │ 112     │ 112
//!    ╰─ (65536, 3, 2)                │               │               │               │         │
//!       ├─ t=1         100.9 ms      │ 138.1 ms      │ 107.6 ms      │ 108.6 ms      │ 100     │ 100
//!       ├─ t=4         136.6 ms      │ 175.1 ms      │ 152 ms        │ 153.6 ms      │ 100     │ 100
//!       ├─ t=8         208.3 ms      │ 254.8 ms      │ 229.5 ms      │ 229.3 ms      │ 104     │ 104
//!       ╰─ t=16        287.7 ms      │ 441.4 ms      │ 365.6 ms      │ 367.7 ms      │ 112     │ 112
//! ```
//!
//! ### File related
//!
//! ```text
//! Timer precision: 100 ns
//! fs                         fastest       │ slowest       │ median        │ mean          │ samples │ iters
//! ├─ deserialize_file        734.7 µs      │ 1.499 ms      │ 952.7 µs      │ 967.7 µs      │ 100     │ 100
//! ├─ deserialize_file_async  758.7 µs      │ 1.702 ms      │ 972.3 µs      │ 1.005 ms      │ 100     │ 100
//! ├─ read_from_file          654.4 µs      │ 1.347 ms      │ 791 µs        │ 834.5 µs      │ 100     │ 100
//! ├─ read_from_file_async    706.8 µs      │ 1.844 ms      │ 831.9 µs      │ 882.3 µs      │ 100     │ 100
//! ├─ serialize_file          723.1 µs      │ 1.108 ms      │ 779.8 µs      │ 803.1 µs      │ 100     │ 100
//! ╰─ serialize_file_async    783 µs        │ 1.267 ms      │ 885.4 µs      │ 912.9 µs      │ 100     │ 100
//! ```
//!
//! ### Path related
//!
//! ```text
//! Timer precision: 100 ns
//! path         fastest       │ slowest       │ median        │ mean          │ samples │ iters
//! ╰─ is_empty  222.6 µs      │ 325.1 µs      │ 231.9 µs      │ 240.6 µs      │ 100     │ 100
//! ```

#![forbid(unsafe_code)]
#![warn(clippy::pedantic, missing_debug_implementations, missing_docs)]

mod crypto;
mod errors;
mod pathext;

use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
    path::Path,
};

pub use crypto::{ArgonParams, derive_key, generate_salt, hash_password, verify_password};
pub use errors::UtilsError;
pub use pathext::PathExt;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

/// Extension trait for [`Option<T>`]
pub trait IntoOptional<T> {
    /// Convert T to [`Option<T>`]
    fn into_optional(self) -> Option<T>;
}

impl<T> IntoOptional<T> for T {
    fn into_optional(self) -> Option<T> {
        Some(self)
    }
}

impl<T> IntoOptional<T> for Option<T> {
    fn into_optional(self) -> Option<T> {
        self
    }
}

/// Deserialize [bitcode] data from a file
///
/// ## Arguments
///
/// * `path` - The path to the file to deserialize from
///
/// ## Returns
///
/// The deserialized data
///
/// ## Errors
///
/// * [`UtilsError::FailedToOpenFile`]: The file could not be opened
/// * [`UtilsError::FailedToReadFile`]: The file could not be read
/// * [`UtilsError::FailedToDeserializeData`]: The data could not be deserialized
///
/// ## Example
///
/// ```rust,no_run
/// use minidb_utils::deserialize_file;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Person {
///     name: String,
///     age: u8,
/// }
///
/// let p: Person = deserialize_file("person.bin").unwrap();
/// ```
pub fn deserialize_file<P, T>(path: P) -> Result<T>
where
    P: AsRef<Path>,
    T: for<'de> Deserialize<'de>,
{
    deserialize_file_impl(path.as_ref())
}

fn deserialize_file_impl<T>(path: &Path) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let file = File::open(path).context(UtilsError::FailedToOpenFile(path.to_path_buf()))?;
    let mut reader = BufReader::new(file);
    let mut data = Vec::new();

    reader
        .read_to_end(&mut data)
        .context(UtilsError::FailedToReadFile(path.to_path_buf()))?;

    let value: T =
        bitcode::deserialize(&data).context(UtilsError::FailedToDeserializeData(data))?;
    Ok(value)
}

/// Deserialize [bitcode] data from a file asynchronously
///
/// ## Arguments
///
/// * `path` - The path to the file to deserialize from
///
/// ## Returns
///
/// The deserialized value
///
/// ## Errors
///
/// * [`UtilsError::FailedToOpenFile`]: The file could not be opened
/// * [`UtilsError::FailedToReadFile`]: The file could not be read
/// * [`UtilsError::FailedToDeserializeData`]: The data could not be deserialized
///
/// ## Example
///
/// ```rust,ignore
/// use minidb_utils::deserialize_file_async;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Person {
///     name: String,
///     age: u8
/// }
///
/// let p: Person = deserialize_file_async("person.bin").await.unwrap();
/// ```
#[cfg(feature = "tokio")]
pub async fn deserialize_file_async<P, T>(path: P) -> Result<T>
where
    P: AsRef<Path>,
    T: for<'de> Deserialize<'de>,
{
    deserialize_file_async_impl(path.as_ref()).await
}

#[cfg(feature = "tokio")]
async fn deserialize_file_async_impl<T>(path: &Path) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    use tokio::io::AsyncReadExt;

    let file = tokio::fs::File::open(path)
        .await
        .context(UtilsError::FailedToOpenFile(path.to_path_buf()))?;
    let mut reader = tokio::io::BufReader::new(file);
    let mut data = Vec::new();

    reader
        .read_to_end(&mut data)
        .await
        .context(UtilsError::FailedToReadFile(path.to_path_buf()))?;

    let value: T =
        bitcode::deserialize(&data).context(UtilsError::FailedToDeserializeData(data))?;
    Ok(value)
}

/// Read a file into a string using a buffer
///
/// ## Arguments
///
/// * `path` - The path to the file to read
///
/// ## Returns
///
/// A string containing the contents of the file
///
/// ## Errors
///
/// * [`UtilsError::FailedToOpenFile`]: The file could not be opened
/// * [`UtilsError::FailedToReadFile`]: The file could not be read
///
/// ## Example
///
/// ```rust,no_run
/// use minidb_utils::read_from_file;
///
/// let str = read_from_file("file.txt").unwrap();
/// ```
pub fn read_from_file<P>(path: P) -> Result<String>
where
    P: AsRef<Path>,
{
    read_from_file_impl(path.as_ref())
}

fn read_from_file_impl(path: &Path) -> Result<String> {
    let file = File::open(path).context(UtilsError::FailedToOpenFile(path.to_path_buf()))?;
    let mut reader = BufReader::new(file);
    let mut data = String::new();

    reader
        .read_to_string(&mut data)
        .context(UtilsError::FailedToReadFile(path.to_path_buf()))?;

    Ok(data)
}

/// Read a file asynchronously into a string using a buffer
///
/// ## Arguments
///
/// * `path` - The path to the file to read
///
/// ## Returns
///
/// A string containing the contents of the file
///
/// ## Errors
///
/// * [`UtilsError::FailedToOpenFile`]: The file could not be opened
/// * [`UtilsError::FailedToReadFile`]: The file could not be read
///
/// ## Example
///
/// ```rust,ignore
/// use minidb_utils::read_from_file_async;
///
/// let str = read_from_file_async("file.txt").await.unwrap();
/// ```
#[cfg(feature = "tokio")]
pub async fn read_from_file_async<P>(path: P) -> Result<String>
where
    P: AsRef<Path>,
{
    read_from_file_async_impl(path.as_ref()).await
}

#[cfg(feature = "tokio")]
async fn read_from_file_async_impl(path: &Path) -> Result<String> {
    use tokio::io::AsyncReadExt;

    let file = tokio::fs::File::open(path)
        .await
        .context(UtilsError::FailedToOpenFile(path.to_path_buf()))?;
    let mut reader = tokio::io::BufReader::new(file);
    let mut data = String::new();

    reader
        .read_to_string(&mut data)
        .await
        .context(UtilsError::FailedToReadFile(path.to_path_buf()))?;

    Ok(data)
}

/// Serialize a value to a file using [bitcode]
///
/// ## Arguments
///
/// * `path` - The path to the file to serialize to
/// * `value` - The value to serialize
///
/// ## Errors
///
/// * [`UtilsError::FailedToCreateTempFile`]: The temp file could not be created
/// * [`UtilsError::FailedToSerializeValue`]: The value could not be serialized
/// * [`UtilsError::FailedToWriteTempFile`]: The temp file could not be written to
/// * [`UtilsError::FailedToFlushTempFile`]: The temp file could not be flushed
/// * [`UtilsError::FailedToGetInnerWriter`]: The inner writer could not be obtained
/// * [`UtilsError::FailedToPersistTempFile`]: The temp file could not be persisted
///
/// ## Example
///
/// ```rust,no_run
/// use minidb_utils::serialize_file;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Person {
///     name: String,
///     age: u8,
/// }
///
/// let p = Person {
///     name: String::from("John Doe"),
///     age: 31,
/// };
///
/// serialize_file("person.bin", &p).unwrap();
/// ```
pub fn serialize_file<P, T>(path: P, value: &T) -> Result<()>
where
    P: AsRef<Path>,
    T: Serialize,
{
    serialize_file_impl(path.as_ref(), value)
}

fn serialize_file_impl<T>(path: &Path, value: &T) -> Result<()>
where
    T: Serialize,
{
    let temp_file = NamedTempFile::new_in(path.parent().unwrap_or(Path::new(".")))
        .context(UtilsError::FailedToCreateTempFile)?;
    let temp_path = temp_file.path().to_path_buf();

    let mut writer = BufWriter::new(temp_file);
    let data = bitcode::serialize(value).context(UtilsError::FailedToSerializeValue)?;

    writer
        .write_all(&data)
        .context(UtilsError::FailedToWriteTempFile(temp_path.clone()))?;
    writer
        .flush()
        .context(UtilsError::FailedToFlushTempFile(temp_path.clone()))?;

    let temp_file = writer
        .into_inner()
        .context(UtilsError::FailedToGetInnerWriter)?;
    temp_file
        .persist(path)
        .context(UtilsError::FailedToPersistTempFile {
            temp: temp_path,
            orig: path.to_path_buf(),
        })?;

    Ok(())
}

/// Serialize a value to a file asynchronously using [bitcode]
///
/// ## Arguments
///
/// * `path` - The path to the file to serialize to
/// * `value` - The value to serialize
///
/// ## Errors
///
/// * [`UtilsError::FailedToCreateTempFile`]: The temp file could not be created
/// * [`UtilsError::FailedToReopenTempFile`]: The temp file could not be reopened
/// * [`UtilsError::FailedToSerializeValue`]: The value could not be serialized
/// * [`UtilsError::FailedToWriteTempFile`]: The temp file could not be written to
/// * [`UtilsError::FailedToFlushTempFile`]: The temp file could not be flushed
/// * [`UtilsError::FailedToPersistTempFile`]: The temp file could not be persisted
///
/// ## Example
///
/// ```rust,ignore
/// use minidb_utils::serialize_file_async;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Person {
///     name: String,
///     age: u8,
/// }
///
/// let p = Person {
///     name: String::from("John Doe"),
///     age: 31,
/// };
///
/// serialize_file_async("person.bin", &p).await.unwrap();
/// ```
#[cfg(feature = "tokio")]
pub async fn serialize_file_async<P, T>(path: P, value: &T) -> Result<()>
where
    P: AsRef<Path>,
    T: Serialize,
{
    serialize_file_async_impl(path.as_ref(), value).await
}

#[cfg(feature = "tokio")]
async fn serialize_file_async_impl<T>(path: &Path, value: &T) -> Result<()>
where
    T: Serialize,
{
    use tokio::io::AsyncWriteExt;

    let temp_file = NamedTempFile::new_in(path.parent().unwrap_or(Path::new(".")))
        .context(UtilsError::FailedToCreateTempFile)?;
    let temp_path = temp_file.path().to_path_buf();
    let mut temp_file_async = tokio::fs::File::from_std(
        temp_file
            .reopen()
            .context(UtilsError::FailedToReopenTempFile(temp_path.clone()))?,
    );
    let mut writer = tokio::io::BufWriter::new(&mut temp_file_async);
    let data = bitcode::serialize(value).context(UtilsError::FailedToSerializeValue)?;

    writer
        .write_all(&data)
        .await
        .context(UtilsError::FailedToWriteTempFile(temp_path.clone()))?;
    writer
        .flush()
        .await
        .context(UtilsError::FailedToFlushTempFile(temp_path.clone()))?;

    temp_file
        .persist(path)
        .context(UtilsError::FailedToPersistTempFile {
            temp: temp_path,
            orig: path.to_path_buf(),
        })?;

    Ok(())
}
