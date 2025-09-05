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
//! │  ├─ (1024, 2, 1)   864.6 µs      │ 1.113 ms      │ 887.8 µs      │ 900.7 µs      │ 100     │ 100
//! │  ├─ (19456, 3, 1)  25.76 ms      │ 30.3 ms       │ 26.62 ms      │ 26.77 ms      │ 100     │ 100
//! │  ╰─ (65536, 3, 2)  96.15 ms      │ 102.2 ms      │ 97.62 ms      │ 97.86 ms      │ 100     │ 100
//! ├─ generate_salt     40.38 ns      │ 41.16 ns      │ 40.77 ns      │ 40.6 ns       │ 100     │ 25600
//! ├─ hash_password                   │               │               │               │         │
//! │  ├─ (1024, 2, 1)   899.6 µs      │ 1.102 ms      │ 920.3 µs      │ 931.2 µs      │ 100     │ 100
//! │  ├─ (19456, 3, 1)  24.59 ms      │ 29.19 ms      │ 25.97 ms      │ 26.15 ms      │ 100     │ 100
//! │  ╰─ (65536, 3, 2)  94.87 ms      │ 100.2 ms      │ 97.22 ms      │ 97.3 ms       │ 100     │ 100
//! ╰─ verify_password                 │               │               │               │         │
//!    ├─ (1024, 2, 1)   900.5 µs      │ 1.344 ms      │ 920.6 µs      │ 941.2 µs      │ 100     │ 100
//!    ├─ (19456, 3, 1)  24.63 ms      │ 32 ms         │ 25.71 ms      │ 25.96 ms      │ 100     │ 100
//!    ╰─ (65536, 3, 2)  95.49 ms      │ 106.2 ms      │ 97.27 ms      │ 97.54 ms      │ 100     │ 100
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
/// Returns an error if:
///
/// * Path does not exist
/// * Failed to open file
/// * Failed to read file
/// * Failed to deserialize
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
/// Returns an error if:
///
/// * Path does not exist
/// * Failed to open file
/// * Failed to read file
/// * Failed to deserialize
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
/// Returns an error if the path does not exist, failed to open or failed to be read
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
/// Returns an error if the path does not exist, failed to open or failed to be read
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
/// Returns an error if:
///
/// * Failed to create temporary file
/// * Failed to serialize value
/// * Failed to write or flush temporary file
/// * Failed to get temporary file from writer
/// * Failed to persist temporary file
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
/// Returns an error if:
///
/// * Failed to create temporary file
/// * Failed to reopen temporary file
/// * Failed to serialize value
/// * Failed to write or flush temporary file
/// * Failed to get temporary file from writer
/// * Failed to persist temporary file
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
